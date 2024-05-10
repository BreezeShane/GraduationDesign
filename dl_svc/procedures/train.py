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
from peft import get_peft_config, get_peft_model, LoraConfig, TaskType

from dl_svc.DataProcess.datasetloader import load_dataset
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

def embedding_cosine_similarity(matrix_1, matrix_2):
    x = matrix_1.mul(matrix_2)
    x = (x - x.min()) / (x.max() - x.min())
    x = (x - 0.5) * 2
    return x

def train(args, carry_on=False):
    """ Train COCA Vit Model. """
    setup_seed(TRAIN_CFG.SEED)

    t_dataloader = load_dataset(args.tset, "train.txt", "class.txt", batch_size=TRAIN_CFG.BATCH_SIZE)
    v_dataloader = load_dataset(args.vset, "val.txt", "class.txt", shuffle=False)

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
        print("\n------------Using LoRA----------------")
        model.print_trainable_parameters()
        print("--------------------------------------\n")

    if args.use_deepspeed:
        parameters = filter(lambda p: p.requires_grad, model.parameters())
        model, optimizer, t_dataloader, lr_scheduler = deepspeed.initialize(
            model=model, model_parameters=parameters, training_data=t_dataloader)
        lr_scheduler.total_num_steps = len(t_dataloader)
    else:
        optimizer = torch.optim.Adam(
            model.parameters(),lr=TRAIN_CFG.LR)
        if TRAIN_CFG.WARM_UP:
            lr_scheduler = CosineAnnealingWarmRestarts(
                optimizer=optimizer,
                T_0=len(t_dataloader),
                T_mult=1,
                # eta_min=1e-6
            )
        else:
            lr_scheduler = CosineAnnealingLR(
                optimizer,
                T_max=len(t_dataloader),
                # eta_min=1e-6
            )

    if carry_on:
        if args.use_lora:
            # config = PeftConfig.from_pretrained(join(args.mod_path, peft_model_id))
            # model = PeftModel.from_pretrained(model, peft_model_id)
            model.load_state_dict(torch.load(join(args.mod_path, peft_model_id)), strict=False)
        else:
            if TRAIN_CFG.EARLY_STOP:
                checkpoint_load_path = join(args.mod_path, 'early_stop_checkpoint.pth')
            else:
                checkpoint_load_path = join(CHECKPOINT_PATH, 'common_checkpoint.pth')
            checkpoint = torch.load(checkpoint_load_path)
            model.load_state_dict(checkpoint['model'])
            optimizer.load_state_dict(checkpoint['opt'])
            lr_scheduler.load_state_dict(checkpoint['schedule'])
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

    model.to(args.device)

    train_epochs_loss = []
    valid_epochs_loss = []
    train_acc = []
    val_acc = []

    for epoch in range(TRAIN_CFG.EPOCHS):
        model.train()
        train_epoch_loss = []
        acc, nums = 0., 0

        for idx, (texts, inputs) in enumerate(tqdm(t_dataloader)):
            inputs = inputs.to(model.local_rank if args.use_deepspeed else args.device)
            texts = texts.to(model.local_rank if args.use_deepspeed else args.device)

            if args.use_deepspeed:
                outputs = model(inputs, texts)
                # The above might be loss = model(inputs, texts) and then the next line should be removed.
                loss = loss_criterion(outputs[0].squeeze(), outputs[1])
                model.backward(loss)
                model.step()    # lr_scheduler.step() would also be executed by this.
            else:
                optimizer.zero_grad()
                outputs = model(inputs, texts)
                loss = loss_criterion(outputs[0].squeeze(), outputs[1])
                loss.backward()
                # torch.nn.utils.clip_grad_norm_(model.parameters(), 2.0) # grad clip
                optimizer.step()
                # Cosine Annealing Learning Rate While not using DeepSpeed
                lr_scheduler.step()

            train_epoch_loss.append(loss.item())
            predictions = embedding_cosine_similarity(outputs[0].squeeze(), outputs[1]).argmax(dim=1)
            labels = texts[:, -1:].reshape(-1)
            acc += sum(predictions == labels).cpu()
            nums += labels.shape[0]

            len_t_dataloader = len(t_dataloader)
            print(f" Epoch: {epoch+1}/{TRAIN_CFG.EPOCHS}, loss={loss.item()}")
            # Print train loss and histogram of parameters' distribution
            writer.add_scalars(
                f"Training_Loss_et_Learning_Rate_Epoch_{epoch+1}", {
                    "Loss": torch.tensor(loss.item(),dtype=torch.float),
                    "LR": torch.tensor(lr_scheduler.get_last_lr(), dtype=torch.float)
                }, idx
            )
            for name, param in model.named_parameters():
                if param.grad is not None:
                  writer.add_histogram(tag=name+'.grad', values=param.grad, global_step=idx)
                writer.add_histogram(tag=name+'.data', values=param.data, global_step=idx)

            if idx > 0 and (idx % (len_t_dataloader // 4) == 0 or idx == len_t_dataloader):
                #=====================valid============================
                if v_dataloader is not None:
                    valid_epoch_loss = []
                    acc, nums = 0., 0
                    with torch.no_grad():
                        model.eval()
                        for vidx, (texts, inputs) in enumerate(tqdm(v_dataloader)):
                            inputs = inputs.to(model.local_rank if args.use_deepspeed else args.device)
                            texts = texts.to(model.local_rank if args.use_deepspeed else args.device)

                            outputs = model(inputs, texts)
                            loss = loss_criterion(outputs[0].squeeze(), outputs[1])

                            valid_epoch_loss.append(loss.item())
                            predictions = embedding_cosine_similarity(outputs[0].squeeze(), outputs[1]).argmax(dim=1)
                            labels = texts[:, -1:].reshape(-1)
                            acc += sum(predictions == labels).cpu()
                            nums += labels.shape[0]
                            #TB Print valid loss
                            writer.add_scalar(f"V_Loss_BS_{idx}_Epoch_{epoch+1}", loss.item(), vidx)
                    v_loss_avg = np.average(valid_epoch_loss)
                    v_acc = 100 * acc / nums
                    valid_epochs_loss.append(v_loss_avg)
                    val_acc.append(v_acc)
                    print("\n---------------------------------------------------------------------------------")
                    print(f"| Epoch: {epoch+1} Iters: {idx} Valid Acc: {v_acc:.3f}%, Avg Loss: {v_loss_avg} |")
                    print("---------------------------------------------------------------------------------\n")
                #==================early stopping======================
                if TRAIN_CFG.EARLY_STOP:
                    early_stopping(
                        valid_epochs_loss[-1],
                        params={
                            'name': model_name,
                            'model_state_dict': model.state_dict(),
                            'optimizer_state_dict': optimizer.state_dict(),
                            'lr_scheduler_state_dict': lr_scheduler.state_dict()
                        },
                        path=CHECKPOINT_PATH
                    )
                    if early_stopping.early_stop:
                        print("Early stopping...")
                        break
                else:
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
                if args.use_lora:
                    model.save_pretrained(join(args.mod_path, peft_model_id))
                else:
                    torch.save({
                            'name': model_name,
                            'model': model.state_dict()
                        },
                        join(
                            args.mod_path,
                            f'{epoch+1}-{idx}-{len_t_dataloader}-model.pt'
                        )
                    )


        t_loss_avg = np.average(train_epoch_loss)
        t_acc = 100 * acc / nums
        train_epochs_loss.append(t_loss_avg)
        train_acc.append(t_acc)
        print("\n------------------------------------------------------------------------")
        print(f"| Epoch {epoch+1} Latest Train Acc = {t_acc:.3f}%, Loss = {t_loss_avg} |")
        print("------------------------------------------------------------------------\n")

    # Print Loss with epochs when epochs more than 1
    if TRAIN_CFG.EPOCHS > 1:
        for epoch in range(len(train_epochs_loss)):
            writer.add_scalar("T_loss_epochs", train_epochs_loss[epoch], epoch+1)
        for epoch in range(len(valid_epochs_loss)):
            writer.add_scalar("V_loss_epochs", valid_epochs_loss[epoch], epoch+1)
    writer.close()
