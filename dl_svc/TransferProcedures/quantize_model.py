import os
import torch
from ModelTransfer.QuantModel import QuantModel, prepare_model, eval_data_distribution, quant_model, save_qmodel
from DataProcess.datasetloader import load_classic_dataset

from TransferProcedures.compile_utils import load_classifier_model

def quantize(args):
    """ Definition to quantize the model from float to int8. (Feature for X86 and ARM.) """
    if None in (args.qset, args.qmodel, args.qsave):
        raise ValueError("All params '--qset, --qmodel, --qsave' are required!")
    if not os.path.isfile(args.qmodel):
        raise ValueError(
            f"The path to model is required, but got {args.mod2cmp} which is not file!")

    model = QuantModel(
        model=load_classifier_model(pretrained_model_path=args.qmodel, device=args.device),
        target=args.qtarget
    )
    model.eval()

    model = prepare_model(model)

    dataloader = load_classic_dataset(data_folder_path=args.qset, record_file="train.txt", shuffle=False)
    eval_data_distribution(model, dataloader)

    model = quant_model(model)

    save_qmodel(model, args.qsave)




