import torch
import numpy as np
from torch import nn
from tqdm import tqdm
from torchinfo import summary
from torch.utils.tensorboard import SummaryWriter
from torch.optim.lr_scheduler import CosineAnnealingWarmRestarts

from ModelTransfer.Classifier import Classifier
from DataProcess.datasetloader import load_classic_dataset
from config import TRANSFER_TRAIN_CFG, CHECKPOINT_PATH, TENSORBOARD_DATA_PATH
from Utils.random_seed import setup_seed

def train(args):
    if None in (args.stset, args.svset, args.smodel):
        raise ValueError("All params '--smodel, --stset, --svset' are required!")

    setup_seed(TRANSFER_TRAIN_CFG.SEED)

    t_dataloader = load_classic_dataset(args.stset, "train.txt", batch_size=TRANSFER_TRAIN_CFG.BATCH_SIZE)
    v_dataloader = load_classic_dataset(args.svset, "val.txt", shuffle=False)

    model = Classifier(
        pretrained_model_path=args.smodel, device=args.device, model_type=args.smodel_type)
    loss_criterion = nn.CrossEntropyLoss()

    optimizer = torch.optim.Adam(
        filter(lambda p: p.requires_grad, model.parameters()), lr=TRANSFER_TRAIN_CFG.LR)
    lr_scheduler = CosineAnnealingWarmRestarts(
        optimizer=optimizer,
        T_0=len(t_dataloader),
        T_mult=1,
        eta_min=1e-6
    )

    writer = SummaryWriter(TENSORBOARD_DATA_PATH)
    #TB Print Model
    print(summary(model, [ (1, 3, 512, 512) ], dtypes=[torch.float], device="cpu"))
    writer.add_graph(model, input_to_model=[
        torch.rand(1, 3, 512, 512)
    ])

    model.to(args.device)

    train_epochs_loss = []
    valid_epochs_loss = []
    train_acc = []
    val_acc = []

    for epoch in range(TRANSFER_TRAIN_CFG.EPOCHS):
        model.train()
        train_epoch_loss = []
        acc, nums = 0., 0

        for idx, (labels, inputs) in enumerate(tqdm(t_dataloader, desc="Training Submodel: ", position=0, leave=True)):
            inputs = inputs.to(args.device)
            labels = labels.to(args.device)

            optimizer.zero_grad()
            outputs = model(inputs)
            predicts = outputs.squeeze(1).argmax(1)
            loss = loss_criterion(predicts, labels)
            loss.backward()
            optimizer.step()
            lr_scheduler.step()

            train_epoch_loss.append(loss.item())
            acc += sum(predicts == labels).cpu()
            nums += labels.shape[0]
            # print(f" Epoch: {epoch+1}/{TRANSFER_TRAIN_CFG.EPOCHS}, loss={loss.item()}")
            # Print train loss and histogram of parameters' distribution
            writer.add_scalar(
                f"Submodel_Training_Loss_Epoch_{epoch+1}",
                torch.tensor(loss.item(),dtype=torch.float), idx
            )
            writer.add_scalar(
                f"Submodel_Learning_Rate_Epoch_{epoch+1}",
                torch.tensor(lr_scheduler.get_last_lr(), dtype=torch.float), idx
            )

            for name, param in model.named_parameters():
                if param.grad is not None:
                  writer.add_histogram(tag=f'submodel.{name}.grad', values=param.grad, global_step=idx)
                writer.add_histogram(tag=f'submodel.{name}.data', values=param.data, global_step=idx)

        len_t_dataloader = len(t_dataloader)
        if idx > 0 and (idx % (len_t_dataloader // 10) == 0 or idx == len_t_dataloader):
            #=====================valid============================
            valid_epoch_loss = []
            v_acc, v_nums = 0., 0
            with torch.no_grad():
                model.eval()
                for vidx, (texts, inputs) in enumerate(tqdm(v_dataloader, desc="Validating submodel: ", position=0, leave=True)):
                    inputs = inputs.to(args.device)
                    texts = texts.to(args.device)

                    outputs = model(inputs)
                    predicts = outputs.squeeze(1).argmax(1)
                    loss = loss_criterion(predicts, labels)

                    valid_epoch_loss.append(loss.item())
                    v_acc += sum(predicts == labels).cpu()
                    v_nums += labels.shape[0]
                    #TB Print valid loss
                    writer.add_scalar(f"Submodel_V_Loss_BS_{idx}_Epoch_{epoch+1}", loss.item(), vidx)
            v_loss_avg = np.average(valid_epoch_loss)
            v_avg_acc = 100 * v_acc / v_nums
            valid_epochs_loss.append(v_loss_avg)
            print("\n----------------------------------------------------------------------------------")
            print(f"| Epoch: {epoch+1} Iters: {idx}, Valid Acc: {v_avg_acc:.3f}%, Avg Loss: {v_loss_avg} |")
            print("----------------------------------------------------------------------------------\n")

            torch.save({
                    'model': model.state_dict(),
                    'opt': optimizer.state_dict(),
                    'scheduler': lr_scheduler.state_dict()
                },
                join(
                    CHECKPOINT_PATH,
                    'common_submodel_checkpoint.pth'
                )
            )

            torch.save(model.state_dict(),
                join(
                    args.mod_path,
                    f'{epoch+1}-{idx}-{len_t_dataloader}-submodel.pt'
                )
            )
        t_loss_avg = np.average(train_epoch_loss)
        t_acc = 100 * acc / nums
        train_epochs_loss.append(t_loss_avg)
        print("\n------------------------------------------------------------------------")
        print(f"| Epoch {epoch+1}, Latest Train Acc = {t_acc:.3f}%, Loss = {t_loss_avg} |")
        print("------------------------------------------------------------------------\n")

    if TRANSFER_TRAIN_CFG.EPOCHS > 1:
        for epoch in range(len(train_epochs_loss)):
            writer.add_scalar("Submodel_T_loss_epochs", train_epochs_loss[epoch], epoch+1)
        for epoch in range(len(valid_epochs_loss)):
            writer.add_scalar("Submodel_V_loss_epochs", valid_epochs_loss[epoch], epoch+1)
    writer.close()
