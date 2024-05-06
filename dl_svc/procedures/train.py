"""
    Training COCA Vit Definition.
"""
import math
import random
from os.path import join
import torch
import numpy as np
import deepspeed
from tqdm import tqdm
from torchinfo import summary
from torch.utils.tensorboard import SummaryWriter
from torch.optim.lr_scheduler import CosineAnnealingLR, CosineAnnealingWarmRestarts
from peft import get_peft_model, LoraConfig, TaskType

from dl_svc.DataProcess.datasetloader import load_image_dataset, load_text_dataset
from dl_svc.COCA.coca_model import coca_vit_b_32, coca_vit_l_14
from dl_svc.COCA.coca_vit_custom import coca_vit_custom
from dl_svc.Loss.contrastive_loss_with_temperature import ContrastiveLossWithTemperature
from dl_svc.Utils.early_stop import EarlyStopping
from dl_svc.config import TRAIN_CFG, TENSORBOARD_DATA_PATH, CHECKPOINT_PATH

def setup_seed(seed):
    """ Set Seed for Replicability. """
    torch.manual_seed(seed)
    torch.cuda.manual_seed_all(seed)
    np.random.seed(seed)
    random.seed(seed)
    torch.backends.cudnn.deterministic = True

def train(args, carry_on=False):
    """ Train COCA Vit Model. """
    setup_seed(TRAIN_CFG.SEED)

    t_img_dataloader = load_image_dataset(args.tset, "train.txt", batch_size=TRAIN_CFG.BATCH_SIZE)
    t_txt_dataloader = load_text_dataset(args.text, "class.txt", batch_size=TRAIN_CFG.BATCH_SIZE)
    v_dataloader = load_image_dataset(args.vset, "val.txt", shuffle=False)

    match args.model_type:
        case 'large':
            model = coca_vit_l_14()
            model_name = 'coca_vit_l_14'
        case 'base':
            model = coca_vit_b_32()
            model_name = 'coca_vit_b_32'
        case 'custom':
            model = coca_vit_custom()
            model_name = 'coca_vit_custom'

    if args.use_lora:
        peft_config = LoraConfig(
            task_type=TaskType.SEQ_2_SEQ_LM,
            inference_mode=False,
            r=8,
            lora_alpha=32,
            lora_dropout=0.1
        )
        model = get_peft_model(model, peft_config)
        peft_model_id = f"{model_name}_{peft_config.peft_type}_{peft_config.task_type}"
        model.print_trainable_parameters()

    if args.use_deepspeed:
        parameters = filter(lambda p: p.requires_grad, model.parameters())
        model, optimizer, t_img_dataloader, lr_scheduler = deepspeed.initialize(
            model=model, model_parameters=parameters, training_data=t_img_dataloader)
        lr_scheduler.total_num_steps = len(t_img_dataloader)
    else:
        optimizer = torch.optim.Adam(
            model.parameters(),lr=TRAIN_CFG.LR)
        if TRAIN_CFG.WARM_UP:
            lr_scheduler = CosineAnnealingWarmRestarts(
                optimizer=optimizer,
                T_0=len(t_img_dataloader),
                T_mult=1,
                # eta_min=1e-6
            )
        else:
            lr_scheduler = CosineAnnealingLR(
                optimizer,
                T_max=len(t_img_dataloader),
                # eta_min=1e-6
            )

    if carry_on:
        checkpoint = torch.load(join(
            args.mod_path if TRAIN_CFG.EARLY_STOP else CHECKPOINT_PATH,
            'checkpoint.pth'
            )
        )
        model.load_state_dict(checkpoint['model'])
        optimizer.load_state_dict(checkpoint['opt'])
        lr_scheduler.load_state_dict(checkpoint['schedule'])
        if args.use_lora:
            # config = PeftConfig.from_pretrained(join(args.mod_path, peft_model_id))
            # model = PeftModel.from_pretrained(model, peft_model_id)
            model.load_state_dict(torch.load(join(args.mod_path, peft_model_id)), strict=False)
        model.eval()

    loss_criterion = ContrastiveLossWithTemperature(
        logit_scale = math.log(1 / 0.07), # DEFAULT_LOGIT_SCALE
        logit_scale_min = math.log(1.0),
        logit_scale_max = math.log(100.0),
    )
    if TRAIN_CFG.EARLY_STOP:
        early_stopping = EarlyStopping(
            TRAIN_CFG.PATIENCE,
            verbose=True,
            delta=0
        )
    writer = SummaryWriter(TENSORBOARD_DATA_PATH)
    #TB Print Model
    print(summary(model, [ (1, 3, 512, 512), (1, 6) ],
        dtypes=[torch.float, torch.long], device="cpu"))
    writer.add_graph(model, input_to_model=[
        torch.rand(1, 3, 512, 512),         # image
        torch.randint(0, 194, size=(1, 6))  # text
    ])

    train_epochs_loss = []
    valid_epochs_loss = []
    train_acc = []
    val_acc = []

    for epoch in range(TRAIN_CFG.EPOCHS):
        model.train()
        train_epoch_loss = []
        acc, nums = 0., 0

        for idx, (texts, inputs) in enumerate(tqdm(t_img_dataloader, t_txt_dataloader)):
            inputs = inputs.to(torch.float32).to(model.local_rank
                                                    if args.use_deepspeed else args.device)
            texts = texts.to(torch.float32).to(model.local_rank
                                                    if args.use_deepspeed else args.device)
            outputs = model(inputs, texts)
            loss = loss_criterion(outputs, texts)

            if args.use_deepspeed:
                model.backward(loss)
                model.step()
                lr_scheduler.step()
            else:
                optimizer.zero_grad()
                model.backward()
                # torch.nn.utils.clip_grad_norm_(model.parameters(), 2.0) # grad clip
                optimizer.step()
            # Cosine Annealing Learning Rate While not using DeepSpeed
            lr_scheduler.step()

            train_epoch_loss.append(loss.item())
            acc += sum(outputs.max(axis=1)[1] == texts).cpu()
            nums += texts.size()[0]

            len_t_dataloader = len(t_img_dataloader)
            if idx % (len_t_dataloader // 1000) == 0:
                print(f"epoch={epoch+1}/{TRAIN_CFG.EPOCHS}, "
                      f"{idx}/{len_t_dataloader} of train, loss={loss.item()}")
                if args.use_lora:
                    model.save_pretrained(join(args.mod_path, peft_model_id))
                torch.save({
                        'name': model_name,
                        'model': model.state_dict()
                    },
                    join(
                        args.mod_path,
                        f'{idx}-{len(t_img_dataloader)}-model.pt'
                    )
                )
                if not TRAIN_CFG.EARLY_STOP:
                    torch.save({
                            'name': model_name,
                            'model': model.state_dict(),
                            'opt': optimizer.state_dict(),
                            'scheduler': lr_scheduler.state_dict()
                        },
                        join(
                            CHECKPOINT_PATH,
                            'common_checkpoint.pth'
                        )
                    )

            #TB Print train loss and histogram of parameters' distribution
            writer.add_scalar(f"T_loss_epoch_{epoch+1}", loss.item(), idx)
            writer.add_scalar(f"learning_rate_epoch_{epoch + 1}", lr_scheduler.get_last_lr(), idx)
            for name, param in model.named_parameters():
                writer.add_histogram(tag=name+'_grad', values=param.grad, global_step=idx)
                writer.add_histogram(tag=name+'_data', values=param.data, global_step=idx)
        t_loss_avg = np.average(train_epoch_loss)
        t_acc = 100 * acc / nums
        train_epochs_loss.append(t_loss_avg)
        train_acc.append(t_acc)
        print(f"train acc = {t_acc:.3f}%, loss = {t_loss_avg}")
        #=====================valid============================
        if v_dataloader is not None:
            with torch.no_grad():
                model.eval()
                valid_epoch_loss = []
                acc, nums = 0., 0
                for idx, (texts, inputs) in enumerate(tqdm(v_dataloader)):
                    inputs = inputs.to(torch.float32).to(args.device)
                    texts = texts.to(torch.float32).to(args.device)
                    outputs = model(inputs)
                    loss = loss_criterion(outputs, texts)

                    valid_epoch_loss.append(loss.item())
                    acc += sum(outputs.max(axis=1)[1] == texts).cpu()
                    nums += texts.size()[0]
                    #TB Print valid loss
                    writer.add_scalar(f"V_loss_epoch_{epoch+1}", loss.item(), idx)


                v_loss_avg = np.average(valid_epoch_loss)
                v_acc = 100 * acc / nums
                valid_epochs_loss.append(v_loss_avg)
                val_acc.append(v_acc)
                print(f"epoch = {epoch+1}, valid acc = {v_acc:.3f}%, loss = {v_loss_avg}")
        #==================early stopping======================
        if TRAIN_CFG.EARLY_STOP:
            early_stopping(
                valid_epochs_loss[-1],
                params={
                    'name': model_name,
                    'model_state_dict': model.state_dict(),
                    'optimizer_state_dict': optimizer.state_dict(),
                },
                path=args.mod_path
            )
            if early_stopping.early_stop:
                print("Early stopping...\n")
                break
        # #====================adjust lr========================
        # lr_adjust = {
        #         2: 5e-5, 4: 1e-5, 6: 5e-6, 8: 1e-6,
        #         10: 5e-7, 15: 1e-7, 20: 5e-8
        #     }
        # if epoch in lr_adjust.keys():
        #     lr = lr_adjust[epoch]
        #     for param_group in optimizer.param_groups:
        #         param_group['lr'] = lr
        #     print('Updating learning rate to {}'.format(lr))

    #TB Print Loss with epochs when epochs more than 1
    if TRAIN_CFG.EPOCHS > 1:
        len_t_loss_epochs = len(train_epochs_loss)
        len_v_loss_epochs = len(valid_epochs_loss)
        for epoch in range(len_t_loss_epochs):
            writer.add_scalar("T_loss_epochs", train_epochs_loss[epoch], epoch+1)
        for epoch in range(len_v_loss_epochs):
            writer.add_scalar("V_loss_epochs", valid_epochs_loss[epoch], epoch+1)
