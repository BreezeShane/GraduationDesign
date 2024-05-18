"""
    Definition of inference and validation procedures.
"""
import math
from os.path import exists, isfile
import numpy as np
from tqdm import tqdm
import torch
from torch import nn

from DataProcess.datasetloader import load_classic_dataset, load_data
from ModelTransfer.Classifier import Classifier

def test(args):
    """ Definition of validation procedure. """
    if args.vset is None:
        raise ValueError("Validate Dataset is needed!")
    t_dataloader = load_classic_dataset(args.svset, "test.txt")

    model = Classifier(
        pretrained_model_path=args.smodel, device=args.device,
        model_type=args.smodel_type
    )
    loss_criterion = nn.CrossEntropyLoss()

    valid_loss = []
    val_acc = []

    with torch.no_grad():
        model.eval()
        valid_epoch_loss = []
        acc, nums = 0., 0

        for _, (label, inputs) in enumerate(tqdm(t_dataloader)):
            inputs = inputs.to(torch.float32).to(args.device)
            label = label.to(torch.float32).to(args.device)
            outputs = model(inputs)
            predicts = outputs.squeeze(1).argmax(1)
            loss = loss_criterion(predicts, labels)

            valid_epoch_loss.append(loss.item())
            acc += sum(predicts == label).cpu()
            nums += label.shape[0]

        v_loss_avg = np.average(valid_epoch_loss)
        v_acc = 100 * acc / nums
        valid_loss.append(v_loss_avg)
        val_acc.append(v_acc)
        print(f"Test Acc = {v_acc:.3f}%, loss = {v_loss_avg}")

def infer(args):
    """ Definition of inference procedure. """
    if args.siset is None:
        raise ValueError("Inference Dataset is needed!")
    i_data_list = load_data(args.siset)
    model = Classifier(
        pretrained_model_path=args.smodel, device=args.device,
        model_type=args.smodel_type
    )

    classes = None
    with open(args.scls_path, encoding="utf-8") as f:
        classes = f.read().splitlines()

    result = []
    with torch.no_grad():
        for _, (img_name, img_input) in enumerate(tqdm(i_data_list)):
            img_input = img_input.to(torch.float32).to(args.device)
            output = model(img_input)

            predict_label = output.argmax(1)
            result.append(
                (img_name, predict_label)
            )
    for img_name, prediction in result:
        print(f"The image `{img_name}` is {classes[prediction]}.")

# def load_whole_model(args):
#     model_path = args.mod_loc
#     if not exists(model_path):
#         raise IOError(f"Trained model not found! Attempting to load: {model_path}")
#     if not isfile(model_path):
#         raise ValueError("Trained model should be a file!")

#     if args.gpu_id is not None:
#         model_dict = torch.load(model_path, map_location=torch.device(f'cuda:{args.gpu_id}'))
#     else:
#         model_dict = torch.load(model_path, map_location=torch.device('cpu'))
#     try:
#         match model_dict['name']:
#             case 'coca_vit_l_14':
#                 model = coca_vit_l_14()
#             case 'coca_vit_b_32':
#                 model = coca_vit_b_32()
#             case 'coca_vit_custom':
#                 model = coca_vit_custom()
#     except KeyError as exc:
#         raise KeyError(f"Not compatible model! Get model here: {model_path}") from exc

#     model.load_state_dict(model_dict['model'])
#     if args.use_lora:
#         if args.lora_path is None:
#             raise ValueError("'lora_path' parameter is needed!")
#         model.load_state_dict(torch.load(args.lora_path), strict=False)
#     model.eval()

#     return model

# def load_single_encoder_model(args):
#     model_path = args.mod_loc
#     if not exists(model_path):
#         raise IOError(f"Trained model not found! Attempting to load: {model_path}")
#     if not isfile(model_path):
#         raise ValueError("Trained model should be a file!")

#     map_location = torch.device(
#         'cpu' if args.gpu_id is None
#         else f'cuda:{args.gpu_id}'
#     )

#     model = vision_transformer(
#         patch_size=32,
#         hidden_dim=768,
#         dim_feedforward=3072,
#         n_layer=12,
#         n_head=12,
#         image_size=512,
#         num_channels=3,
#         activation=nn.GELU,
#         transformer_dropout=0.0,
#         patch_embed_dropout_prob=0.0,
#         layer_norm_eps=1e-5,
#         final_layer_norm_eps=None,
#         norm_first=True,
#         include_cls_embed=False,
#         drop_path_rate=None,
#         patch_drop_rate=None,
#     )

#     trained_model = torch.load(model_path, map_location=map_location)['model']
#     model_dict = model.state_dict()
#     load_dict = { k:v for k, v in trained_model.items() if k in model_dict.keys()}
#     model_dict.update(load_dict)
#     model.load_state_dict(model_dict)
#     if args.use_lora:
#         if args.lora_path is None:
#             raise ValueError("'lora_path' parameter is needed!")
#         model.load_state_dict(torch.load(args.lora_path), strict=False)
#     model.eval()

#     return model
