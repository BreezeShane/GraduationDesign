"""
    Definition of inference and validation procedures.
"""
import math
from os.path import exists, isfile
import numpy as np
from tqdm import tqdm
import torch

from dl_svc.DataProcess.datasetloader import load_image_dataset, load_data
from dl_svc.Loss.contrastive_loss_with_temperature import ContrastiveLossWithTemperature
from dl_svc.COCA.coca_model import coca_vit_b_32, coca_vit_l_14
from dl_svc.COCA.coca_vit_custom import coca_vit_custom

def validate(args):
    """ Definition of validation procedure. """
    if args.vset is None:
        raise ValueError("Validate Dataset is needed!")
    v_dataloader = load_image_dataset(args.vset, "valid.txt")

    model = __load_model(args=args)
    loss_criterion = ContrastiveLossWithTemperature(
        logit_scale = math.log(1 / 0.07), # DEFAULT_LOGIT_SCALE
        logit_scale_min = math.log(1.0),
        logit_scale_max = math.log(100.0),
    )

    valid_loss = []
    val_acc = []

    with torch.no_grad():
        valid_epoch_loss = []
        acc, nums = 0., 0

        for _, (label, inputs) in enumerate(tqdm(v_dataloader)):
            inputs = inputs.to(torch.float32).to(args.device)
            label = label.to(torch.float32).to(args.device)
            outputs = model(inputs)
            loss = loss_criterion(outputs, label)

            valid_epoch_loss.append(loss.item())
            acc += sum(outputs.max(axis=1)[1] == label).cpu()
            nums += label.size()[0]

        v_loss_avg = np.average(valid_epoch_loss)
        v_acc = 100 * acc / nums
        valid_loss.append(v_loss_avg)
        val_acc.append(v_acc)
        print(f"Valid Acc = {v_acc:.3f}%, loss = {v_loss_avg}")

def inference(args):
    """ Definition of inference procedure. """
    if args.iset is None:
        raise ValueError("Inference Dataset is needed!")
    i_data_list = load_data(args.iset)
    model = __load_model(args=args)
    loss_criterion = ContrastiveLossWithTemperature(
        logit_scale = math.log(1 / 0.07), # DEFAULT_LOGIT_SCALE
        logit_scale_min = math.log(1.0),
        logit_scale_max = math.log(100.0),
    )

    result = []
    with torch.no_grad():
        for _, (img_name, img_input) in enumerate(tqdm(i_data_list)):
            img_input = img_input.to(torch.float32).to(args.device)
            output = model(img_input)

            label = torch.argmax(output, 1)
            loss = loss_criterion(output, label)
            result.append(
                (img_name, label, loss)
            )
    print(result)
    return result


def __load_model(args):
    model_path = args.mod_loc
    if not exists(model_path):
        raise IOError(f"Trained model not found! Attempting to load: {model_path}")
    if not isfile(model_path):
        raise ValueError("Trained model should be a file!")

    if args.gpu_id is not None:
        model_dict = torch.load(model_path, map_location=torch.device(f'cuda:{args.gpu_id}'))
    else:
        model_dict = torch.load(model_path, map_location=torch.device('cpu'))
    try:
        match model_dict['name']:
            case 'coca_vit_l_14':
                model = coca_vit_l_14()
            case 'coca_vit_b_32':
                model = coca_vit_b_32()
            case 'coca_vit_custom':
                model = coca_vit_custom()
    except KeyError as exc:
        raise KeyError(f"Not compatible model! Get model here: {model_path}") from exc

    model.load_state_dict(model_dict['model'])
    if args.use_lora:
        if args.lora_path is None:
            raise ValueError("'lora_path' parameter is needed!")
        model.load_state_dict(torch.load(args.lora_path), strict=False)
    model.eval()

    return model
