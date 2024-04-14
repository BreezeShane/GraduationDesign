import torch
from tqdm import tqdm
from os.path import join
from torchsummary import summary
from torch.optim.lr_scheduler import CosineAnnealingLR, CosineAnnealingWarmRestarts

from dl_svc.datasetloader import load_dataset
from dl_svc.COCA.coca_model import coca_vit_b_32, coca_vit_l_14, coca_vit
from dl_svc.Loss.contrastive_loss_with_temperature import ContrastiveLossWithTemperature
from dl_svc.Utils.early_stop import EarlyStopping

def train(args, config, custom_net=False, carry_on=False):
    t_dataloader = load_dataset(args.tset, 
        batch_size=config.getint('batch_size'))
    v_dataloader = None
    if args.vset is not None:
        v_dataloader = load_dataset(args.vset)
    
    if custom_net:
        model = coca_vit()

        model_name = 'custom_coca_vit'
        pass
    else:
        model = coca_vit_l_14()
        model_name = 'coca_vit_l_14'
        # model = coca_vit_b_32()
        # model_name = 'coca_vit_b_32'
    optimizer = torch.optim.Adam(
        model.parameters(),lr=config.getfloat('learning_rate'))
    if carry_on:
        checkpoint = torch.load(join(args.mod_path, 'checkpoint.pth'))
        model.load_state_dict(checkpoint['model_state_dict'])
        optimizer.load_state_dict(checkpoint['optimizer_state_dict'])
        model.eval()


    if config.getboolean('enable_warm_up'):
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
    loss_criterion = ContrastiveLossWithTemperature(
        logit_scale = math.log(1 / 0.07), # DEFAULT_LOGIT_SCALE
        logit_scale_min = math.log(1.0),
        logit_scale_max = math.log(100.0),
    )
    if config.getboolean('enable_early_stop'):
        early_stopping = EarlyStopping(
            config.getint('patience'), 
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

    for epoch in range(config.getint('epochs')):
        model.train()
        train_epoch_loss = []
        acc, nums = 0., 0

        for idx, (inputs, label) in enumerate(tqdm(t_dataloader)):
            inputs = inputs.to(torch.float32).to(args.device)
            label = label.to(torch.float32).to(args.device)
            outputs = model(inputs)
            optimizer.zero_grad()
            loss = loss_criterion(outputs, label)
            loss.backward()
            # torch.nn.utils.clip_grad_norm_(model.parameters(), 2.0) # grad clip
            optimizer.step()
            # Cosine Annealing Learning Rate
            lr_scheduler.step()

            train_epoch_loss.append(loss.item())
            acc += sum(outputs.max(axis=1)[1] == label).cpu()
            nums += label.size()[0]
            
            if idx % (len(t_dataloader) // 1000) == 0:
                print("epoch={}/{}, {}/{} of train, loss={}".format(
                    epoch+1, config.getint('epochs'), idx, len(t_dataloader), loss.item()))
                torch.save({
                        'name': model_name,
                        'model': model.state_dict()
                    },
                    join(
                        args.mod_path,
                        f'{idx}-{len(t_dataloader)}-model.pt'
                    )
                )
            
            #TB Print train loss and histogram of parameters' distribution
            writer.add_scalar(f"T_loss_epoch_{epoch+1}", loss.item(), idx)
            writer.add_scalar(f"learning_rate_epoch_{epoch+1}", lr_scheduler.get_last_lr(), idx)
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
        with torch.no_grad():
            model.eval()
            valid_epoch_loss = []
            acc, nums = 0., 0
            
            for idx,(label, inputs) in enumerate(tqdm(valid_dataloader)):
                inputs = inputs.to(torch.float32).to(args.device)
                label = label.to(torch.float32).to(args.device)
                outputs = model(inputs)
                loss = loss_criterion(outputs, label)

                valid_epoch_loss.append(loss.item())
                acc += sum(outputs.max(axis=1)[1] == label).cpu()
                nums += label.size()[0]
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
        if config.getboolean('enable_early_stop'):
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
    if config.getint('epochs') > 1:
        for epoch in range(len(train_epochs_loss)):
            writer.add_scalar("T_loss_epochs", train_epochs_loss[epoch], epoch+1)
        for epoch in range(len(valid_epochs_loss)):
            writer.add_scalar("V_loss_epochs", valid_epochs_loss[epoch], epoch+1)


