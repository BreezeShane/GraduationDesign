import torch
from tqdm import tqdm
from os.path import join
from torchsummary import summary
from torch.optim.lr_scheduler import CosineAnnealingLR, CosineAnnealingWarmRestarts
from peft import get_peft_config, get_peft_model, LoraConfig, TaskType

from dl_svc.datasetloader import load_dataset
from dl_svc.COCA.coca_model import coca_vit_b_32, coca_vit_l_14
from dl_svc.COCA.coca_vit_custom import coca_vit_custom
from dl_svc.Loss.contrastive_loss_with_temperature import ContrastiveLossWithTemperature
from dl_svc.Utils.early_stop import EarlyStopping
from dl_svc.train_config import TRAIN_CFG

def train(args, carry_on=False):
    t_dataloader = load_dataset(args.tset, 
        batch_size=TRAIN_CFG.BATCH_SIZE)

    v_dataloader = load_dataset(args.vset, "val.txt", shuffle=False) if args.vset is not None else None
    
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
        model, optimizer, t_dataloader, lr_scheduler = deepspeed.initialize(
            model=model, model_parameters=parameters, training_data=t_dataset)
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
    rand_input = torch.rand(1, 3, 224, 224)
    writer.add_graph(model, 
        input_to_model=rand_input)
    print(summary(net, rand_input, device="cpu"))

    train_epochs_loss = []
    valid_epochs_loss = []
    train_acc = []
    val_acc = []

    for epoch in range(TRAIN_CFG.EPOCHS):
        model.train()
        train_epoch_loss = []
        acc, nums = 0., 0

        for idx, (labels, inputss) in enumerate(tqdm(t_dataloader)):
            inputs = inputs.to(torch.float32).to(model.local_rank if args.use_deepspeed else args.device)
            labels = labels.to(torch.float32).to(model.local_rank if args.use_deepspeed else args.device)
            outputs = model(inputs)
            loss = loss_criterion(outputs, labels)

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
            acc += sum(outputs.max(axis=1)[1] == labels).cpu()
            nums += labels.size()[0]
            
            if idx % (len(t_dataloader) // 1000) == 0:
                print("epoch={}/{}, {}/{} of train, loss={}".format(
                    epoch+1, TRAIN_CFG.EPOCHS, idx, len(t_dataloader), loss.item()))
                if args.use_lora:
                    model.save_pretrained(join(args.mod_path, peft_model_id))
                torch.save({
                        'name': model_name,
                        'model': model.state_dict()
                    },
                    join(
                        args.mod_path,
                        f'{idx}-{len(t_dataloader)}-model.pt'
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
            
        E_t_loss = np.average(train_epoch_loss)
        Acc_t = 100 * acc / nums
        train_epochs_loss.append(E_t_loss)
        train_acc.append(Acc_t)
        print(
            "train acc = {:.3f}%, loss = {}"
            .format(Acc_t, E_t_loss))
        #=====================valid============================
        if v_dataloader is not None:
            with torch.no_grad():
                model.eval()
                valid_epoch_loss = []
                acc, nums = 0., 0
                
                for idx, (labels, inputs) in enumerate(tqdm(v_dataloader)):
                    inputs = inputs.to(torch.float32).to(args.device)
                    labels = labels.to(torch.float32).to(args.device)
                    outputs = model(inputs)
                    loss = loss_criterion(outputs, labels)

                    valid_epoch_loss.append(loss.item())
                    acc += sum(outputs.max(axis=1)[1] == labels).cpu()
                    nums += labels.size()[0]
                    #TB Print valid loss
                    writer.add_scalar(f"V_loss_epoch_{epoch+1}", loss.item(), idx)


                E_v_loss = np.average(valid_epoch_loss)
                Acc_v = 100 * acc / nums
                valid_epochs_loss.append(E_v_loss)
                val_acc.append(Acc_v)
                print(
                    "epoch = {}, valid acc = {:.3f}%, loss = {}"
                    .format(epoch+1, Acc_v, E_v_loss)
                )
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
        for epoch in range(len(train_epochs_loss)):
            writer.add_scalar("T_loss_epochs", train_epochs_loss[epoch], epoch+1)
        for epoch in range(len(valid_epochs_loss)):
            writer.add_scalar("V_loss_epochs", valid_epochs_loss[epoch], epoch+1)
