"""
The procedure to process command arguments from user.
"""
import os
import ctypes
from argparse import ArgumentParser
from tvm.target import Target

import torch
import warnings

from CoCaProcedures.train import train as coca_train
from TransferProcedures.train import train as submodel_train
from TransferProcedures.infer_et_test import test as submodel_test, infer as submodel_infer
from CoCaProcedures.compile_model import compile_model
from config import CHECKPOINT_PATH, TENSORBOARD_DATA_PATH, OS_NAME

def init_dirs(arguments):
    """ Initialize the directories needed. """
    __create_dir(arguments.mod_path)
    __create_dir(CHECKPOINT_PATH)
    __create_hidden_dir(TENSORBOARD_DATA_PATH)

def __create_dir(dir_path):
    if not os.path.exists(dir_path):
        os.mkdir(dir_path)

def __create_hidden_dir(dir_path):
    if OS_NAME == "linux":
        paths = dir_path.split('/')
        paths[-1] = "." + paths[-1]
        dir_path = os.path.join(*paths)
    if not os.path.exists(dir_path):
        if OS_NAME =="windows":
            _ret = ctypes.windll.kernel32.SetFileAttributesW(
                dir_path, 0x02) # FILE_ATTRIBUTE_HIDDEN = 0x02
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
    if OS_NAME != "windows" and OS_NAME != "linux":
        raise OSError("Not supported operation system!")
    warnings.filterwarnings("ignore", category=torch.jit.TracerWarning)

    parser = ArgumentParser()

    parser.add_argument('mode', type=str,
        choices=[
            'train', 'compile_model', 'show_graphs', 'list_targets'
            'train_submodel', 'infer_submodel', 'test_submodel',
            'ciallo'
        ],
        help="Toggle to the mode you want to run.")
    parser.add_argument('--device', '-d', help="The GPU id to use.")


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

    # mode compile_model
    compile_group_parser = parser.add_argument_group(title='Compile Mode')
    # -------------------------------------------------------------------- #
    compile_group_parser.add_argument('--model_path', dest='mod2cmp', type=str,
        help="The path to the model for compiling.")
    # see https://tvm.apache.org/docs/reference/api/python/target.html#module-tvm.target for detals.
    compile_group_parser.add_argument('--target', type=str, choices=Target.list_kinds(),
        help="Select the compile target. Run `python manager.py list_targets` for details.")
    compile_group_parser.add_argument('--save_package_path', dest='pkg_path', type=str,
        help="The path to save the compiled model.")
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
    init_dirs(args)

    match args.mode:
        case 'train':
            coca_train(args, carry_on=args.carry_on)
        case 'compile_model':
            compile_model(args)
        case 'show_graphs':
            pass
        case 'list_targets':
            list_targets()
        case 'train_submodel':
            submodel_train(args)
        case 'infer_submodel':
            submodel_infer(args)
        case 'test_submodel':
            submodel_test(args)
        case _:
            print("Ciallo～(∠・ω< )⌒☆")
