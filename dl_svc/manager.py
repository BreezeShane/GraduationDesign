"""
The procedure to process command arguments from user.
"""
import os
from argparse import ArgumentParser
from tvm.target import Target

import torch
import warnings

from CoCaProcedures.train import train as coca_train
from TransferProcedures.train import train as submodel_train
from TransferProcedures.infer_et_test import test as submodel_test, infer as submodel_infer
from TransferProcedures.quantize_model import quantize as quantize_submodel
from TransferProcedures.compile_model import compile_model, test_module
from TransferProcedures.compile_utils import transfer_classifier_model
from config import CHECKPOINT_PATH, TENSORBOARD_DATA_PATH, COMPILED_MODEL_DIR, QUANTIZED_MODEL_DIR

def init_dirs(dir_list: list):
    """ Initialize the directories needed. """
    for dir_path in dir_list:
        if not os.path.exists(dir_path):
            os.mkdir(dir_path)

def check_device(arguments):
    """ Check if the device going to use available. """
    if arguments.device is not None and torch.cuda.is_available():
        gpu_id = arguments.device
        if gpu_id < '0' or gpu_id > '9':
            raise TypeError(f"Expected integer index representing GPU ID, but got '{gpu_id}'.\n"
                             "Tips: Expected device parameter example: '-d 0'.")
        arguments.device = torch.device(f"cuda:{gpu_id}")
    else:
        arguments.device = torch.device("cpu")

## Not applied caused by the lack of GPUs.
## The function will be used on Multi-GPU Training implementation.
# def check_multi_devices(arguments):
#     """ Check multi-devices for data paralell traning. """
#     if arguments.device is not None and torch.cuda.is_available():
#         device_list = arguments.device.split(',')
#         for i in device_list:
#             if i < '0' or i > '9':
#                 raise TypeError(
#                   f"Expected integer index representing GPU ID, but got '{i}'."
#                   "Tips: Expected device parameter example: '-d 0', '-d 0,1,2', etc.")
#         arguments.device = [
#             torch.device(f"cuda:{cuda_index}")
#             for cuda_index in device_list
#         ]
#     else:
#         arguments.device = [torch.device("cpu")]

def list_targets():
    """ List all targets TVM supported. """
    supported_tagets = Target.list_kinds()
    print("/*==== Supported Targets ====*/")
    for target in supported_tagets:
        print(" - " + target)
    print("/*===========================*/")

if __name__ == '__main__':
    warnings.filterwarnings("ignore", category=torch.jit.TracerWarning)

    parser = ArgumentParser()

    parser.add_argument('mode', type=str,
        choices=[
            'train', 'compile_model', 'test_modules',
            'show_graphs', 'list_targets',
            'train_submodel', 'infer_submodel', 'test_submodel', 'quantize_submodel',
            'transfer_classifier_model',
            'ciallo'
        ],
        help="Toggle to the mode you want to run.")
    parser.add_argument('--device', '-d', help="The GPU id to use.")
    parser.add_argument('--log', dest='log_dir', default=TENSORBOARD_DATA_PATH, help='Run tensorboard to show graphs.')


    # mode train
    train_group_parser = parser.add_argument_group(title='Train Mode')
    # -------------------------------------------------------------------- #
    train_group_parser.add_argument('--tset', type=str,
        help="The path to train dataset folder for loading.")
    train_group_parser.add_argument('--text', type=str,
        help="The path to folder containing species dictionary for train dataset.")
    train_group_parser.add_argument('--cls', dest='cls_path', type=str,
        help="The path to text file containing species' name in train dataset.")
    train_group_parser.add_argument('--vset', type=str,
        help=
        "The path to validate dataset folder for loading. And enable to validate while training.")
    train_group_parser.add_argument('--network', dest='model_type', type=str, default='custom',
        choices=['base', 'large', 'custom'],
        help="Toggle the size mode of network.")
    train_group_parser.add_argument('--save_model_path', dest='mod_path',
        default='./models', type=str,
        help="The path for the trained model you save")
    train_group_parser.add_argument('--carry_on', action='store_true', default=False,
        help="Set True to continue training the model.")
    train_group_parser.add_argument('--use_deepspeed', action='store_true', default=False,
        help="Apply to use DeepSpeed.")
    train_group_parser.add_argument('--use_lora', action='store_true', default=False,
        help="Apply to use Low-Rank Adaptation, which known as LoRA.")

    # mode quantize_model
    quantize_group_parser = parser.add_argument_group(title='Quantize Mode')
    # -------------------------------------------------------------------- #
    quantize_group_parser.add_argument('--qset', type=str,
        help="The path to folder containing training dataset for evaluating data distribution to quantize the model. ")
    quantize_group_parser.add_argument('--qmodel', type=str,
        help="The path to the pretrained model for quantization. ")
    quantize_group_parser.add_argument('--qtarget', type=str, choices=["x86", "arm"], default="x86",
        help="The target platform to run as server. ")
    quantize_group_parser.add_argument('--qsave', type=str, default=QUANTIZED_MODEL_DIR,
        help="The path to folder for saving the quantized Pytorch model and quantized TorchScript model. ")

    # mode compile_model
    compile_group_parser = parser.add_argument_group(title='Compile Mode')
    # -------------------------------------------------------------------- #
    compile_group_parser.add_argument('--model_path', dest='mod2cmp', type=str,
        help="The path to the model for compiling.")
    # see https://tvm.apache.org/docs/reference/api/python/target.html#module-tvm.target for detals.
    compile_group_parser.add_argument('--target', type=str, choices=Target.list_kinds(),
        help="Select the compile target. Run `python manager.py list_targets` for details.")
    compile_group_parser.add_argument('--save_path', type=str, default=COMPILED_MODEL_DIR,
        help="The path to save the compiled model.")
    compile_group_parser.add_argument('--ts', type=str,
        help="Set True if pretrained model is TorchScript model.")
    compile_group_parser.add_argument('--tune_mode', dest='tune',
        action='store_true', default=False,
        help="Enable to tune the model while compiling.")
    compile_group_parser.add_argument('--continue_compile', action='store_true', default=False,
        help="Enable to continue tuning the model.")
    compile_group_parser.add_argument('--enable_autoscheduler',
        dest='autoscheduler', action='store_true', default=False,
        help="Enable to apply the later version TVM.")
    compile_group_parser.add_argument('--set_trails', dest='trails', default=10000, type=int,
        help="Set the search time. Only needed on compiling large model.")
    compile_group_parser.add_argument('--set_timeout', dest='timeout', default=10, type=int,
        help="Set the timeout on each tuning step. Only needed on compiling large model.")

    submodel_group_parser = parser.add_argument_group(title='Submodel Mode')
    # -------------------------------------------------------------------- #
    submodel_group_parser.add_argument('--smodel', type=str,
        help="The path to pretrained model.pt")
    submodel_group_parser.add_argument('--stset', type=str,
        help="The path to training dataset folder.")
    submodel_group_parser.add_argument('--svset', type=str,
        help="The path to validating dataset folder.")
    submodel_group_parser.add_argument('--smodel_type', type=str, default="normal",
        choices=["normal", "lora", "deepspeed"],
        help="The pretrained model type.")
    submodel_group_parser.add_argument('--stest', type=str,
        help="The path to test dataset folder.")
    submodel_group_parser.add_argument('--siset', type=str,
        help="The path to folder containing images to infer.")
    submodel_group_parser.add_argument('--scls', dest='scls_path', type=str,
        help="The path to text file containing species' name in train dataset.")

    args = parser.parse_args()

    check_device(args)
    init_dirs([
        args.mod_path,
        CHECKPOINT_PATH,
        TENSORBOARD_DATA_PATH,
        COMPILED_MODEL_DIR,
        QUANTIZED_MODEL_DIR
    ])

    if args.mode == 'train':
            coca_train(args, carry_on=args.carry_on)
    elif args.mode == 'compile_model':
            compile_model(args)
    elif args.mode == 'test_modules':
            test_module(args)
    elif args.mode == 'show_graphs':
        os.environ['CRYPTOGRAPHY_OPENSSL_NO_LEGACY'] = '1'
        os.system(f"tensorboard --logdir {args.log_dir}")
    elif args.mode == 'list_targets':
        list_targets()
    elif args.mode == 'train_submodel':
        submodel_train(args)
    elif args.mode == 'infer_submodel':
        submodel_infer(args)
    elif args.mode == 'test_submodel':
        submodel_test(args)
    elif args.mode == 'quantize_submodel':
        quantize_submodel(args)
    elif args.mode == 'transfer_classifier_model':
        transfer_classifier_model(args)
    elif args.mode == 'ciallo':
            print("Ciallo～(∠・ω< )⌒☆")
