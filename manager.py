import os
import torch
import ctypes
import platform
from tvm.target import Target
from configparser import ConfigParser
from argparse import ArgumentParser, FileType
from torch.utils.tensorboard import SummaryWriter
from dl_svc.procedures.train import train
from dl_svc.procedures.infer_et_valid import validate, inference

def init_dirs():
    create_dir(args.mod_path)
    create_dir(CHECKPOINT_PATH)
    create_hidden_dir(TENSORBOARD_DATA_PATH)

def create_dir(dir_path):
    if not os.path.exists(dir_path):
        os.mkdir(dir_path)

def create_hidden_dir(dir_path):
    if OS_NAME == "linux":
        paths = dir_path.split('/')
        paths[-1] = "." + paths[-1]
        dir_path = os.path.join(*paths)
    if not os.path.exists(dir_path):
        if OS_NAME =="windows":
            FILE_ATTRIBUTE_HIDDEN = 0x02
            ret = ctypes.windll.kernel32.SetFileAttributesW(
                dir_path, FILE_ATTRIBUTE_HIDDEN)
        os.mkdir(dir_path)

def check_device():
    if args.device is not None and torch.cuda.is_available():
        gpu_id = args.device
        if gpu_id < '0' or gpu_id > '9':
            raise TypeError(f"Expected integer index representing GPU ID, but got '{i}'. \nTips: Expected device parameter example: '-d 0'.")
        args.device = [torch.device(f"cuda:{gpu_id}") ]
    else:
        args.device = [torch.device("cpu")]

### Not applied caused by the lack of GPUs.
### The function will be used on Multi-GPU Training implementation.
# def check_multi_devices():
#     if args.device is not None and torch.cuda.is_available():
#         device_list = args.device.split(',')
#         for i in device_list:
#             if i < '0' or i > '9':
#                 raise TypeError(f"Expected integer index representing GPU ID, but got '{i}'. \nTips: Expected device parameter example: '-d 0', '-d 0,1,2', etc.")
#         args.device = [
#             torch.device(f"cuda:{cuda_index}") 
#             for cuda_index in device_list
#         ]
#     else:
#         args.device = [torch.device("cpu")]

def list_targets():
    supported_tagets = Target.list_kinds()
    print("/*==== Supported Targets ====*/")
    for target in supported_tagets:
        print(" - " + target)
    print("/*===========================*/")
    pass

if __name__ == '__main__':
    DEFAULT_CONFIG_PATH = 'dl_svc/default/cfg.ini'
    CHECKPOINT_PATH = 'checkpoint/'
    TENSORBOARD_DATA_PATH = 'log'
    OS_NAME = platform.system().lower()

    if OS_NAME != "windows" and OS_NAME != "linux":
        raise OSError("Not supported operation system!")

    parser = ArgumentParser()

    parser.add_argument('mode', type=str, 
        choices=['train', 'valid', 'compile_model', 'infer', 'show_graphs', 'list_targets'], 
        help="Toggle to the mode you want to run.")
    # Optional for Train, Valid and Infer
    parser.add_argument('--gpu_id', type=int,
        help='The GPU ID loading model. Use single GPU.')
    parser.add_argument('--use_lora', action='store_true', default=False,
        help="Apply to use Low-Rank Adaptation, which known as LoRA.")
    parser.add_argument('--lora_path', type=str, 
        help="Load model while applying LoRA, so please enable use_lora and give the path to LoRA models.")
    # Optional for Valid and Infer
    parser.add_argument('--load_model_path', dest='mod_loc',
        default='./models', type=str, 
        help="The path to tained model.")
    
    # mode train
    train_group_parser = parser.add_argument_group(title='Train Mode')
    # -------------------------------------------------------------------- #
    train_group_parser.add_argument('--tset', type=str, 
        help="The path to train dataset for loading.")
    train_group_parser.add_argument('--validate', dest='vset', type=str, 
        help="The path to validate dataset for loading. And enable to validate while training.")
    train_group_parser.add_argument('--network', dest='model_type', type=str, default='large',
        choices=['base', 'large', 'custom'], 
        help="Toggle the size mode of network.")
    train_group_parser.add_argument('--device', '-d', 
        help="The GPU id to use.")
    train_group_parser.add_argument('--save_model_path', dest='mod_path', default='./models', type=str, 
        help="The path for the trained model you save")
    train_group_parser.add_argument('--carry_on', action='store_true', default=False, 
        help="Set True to continue training the model.")
    train_group_parser.add_argument('--use_deepspeed', action='store_true', default=False, 
        help="Apply to use DeepSpeed.")

    # mode valid
    valid_group_parser = parser.add_argument_group(title='Validate Mode')
    # -------------------------------------------------------------------- #
    valid_group_parser.add_argument('--vset', type=str, 
        help="The path to validate dataset for loading.")

    # mode infer
    infer_group_parser = parser.add_argument_group(title='Inference Mode')
    # -------------------------------------------------------------------- #
    infer_group_parser.add_argument('--images', dest='iset', type=str, 
        help="The path to the folder containing images to inference.")

    # mode compile_model
    compile_group_parser = parser.add_argument_group(title='Compile Mode')
    # -------------------------------------------------------------------- #
    compile_group_parser.add_argument('--model_path', dest='mod2cmp', type=str, 
        help="The path to the model for compiling.")
    # see https://tvm.apache.org/docs/reference/api/python/target.html#module-tvm.target for detals.
    compile_group_parser.add_argument('--target', dest='target', type=str, choices=Target.list_kinds(), 
        help="Select the compile target.")
    compile_group_parser.add_argument('--save_package_path', dest='pkg_path', type=str, 
        help="The path to save the compiled model.")
    compile_group_parser.add_argument('--tune_mode', dest='tune', action='store_true', default=False, 
        help="Enable to tune the model while compiling.")
    compile_group_parser.add_argument('--continue_compile', action='store_true', default=False, 
        help="Enable to continue tuning the model.")
    compile_group_parser.add_argument('--enable_autoscheduler', dest='autoscheduler', action='store_true', default=False, 
        help="Enable to apply the later version TVM.")
    compile_group_parser.add_argument('--set_trails', dest='trails', default=10000, type=int, 
        help="Set the search time. Only needed on compiling large model.")
    compile_group_parser.add_argument('--set_timeout', dest='timeout', default=10, type=int, 
        help="Set the timeout on each tuning step. Only needed on compiling large model.")
    # mode valid (Optional for mode train)

    args = parser.parse_args()

    check_device()    
    init_dirs()

    match args.mode:
        case 'train':
            train(args, carry_on=args.carry_on)
        case 'valid':
            validate(args=args)
        case 'infer':
            result = inference(args=args)
        case 'compile_model':
            compile_model(args=args)
        case 'show_graphs':
            pass
        case 'list_targets':
            list_targets()