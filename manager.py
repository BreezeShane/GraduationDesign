import os
import torch
from tvm.target import Target
from configparser import ConfigParser
from argparse import ArgumentParser, FileType
from ctypes.windll.kernel32 import SetFileAttributesW
from torch.utils.tensorboard import SummaryWriter
from dl_svc.procedures.train import train
from dl_svc.procedures.infer_et_valid import validate, inference

if __name__ == '__main__':
    DEFAULT_CONFIG_PATH = './dl_svc/default/cfg.ini'
    CHECKPOINT_PATH = './checkpoint/'
    TENSORBOARD_DATA_PATH = './.log'

    parser = ArgumentParser()

    parser.add_argument('mode', type=str, 
        choices=['train', 'valid', 'compile_model', 'infer', 'show_graphs'])
    
    # mode train
    parser.add_argument('train_dataset', dest='tset', type=str)
    parser.add_argument('--device', '-d')
    parser.add_argument('--load_config', '-l', action='append', 
        dest='f_cfg', help='Load other config file.', 
        type=FileType('rb'))
    parser.add_argument('--save_model_path', dest='mod_path',
        default='./models', type=str)
    parser.add_argument('--custom_net', action='store_true', default=False, 
        help='Enable to adjust the struct of network. Will load Config File named "cfg.ini".')
    parser.add_argument('--carry_on', action='store_true', default=False)
    parser.add_argument('--use_deepspeed', action='store_ture', default=False)
    
    # mode valid (Optional for mode train)
    parser.add_argument('--validate_dataset', '-v', dest='vset', type=str)

    # mode infer (Also for mode valid)
    parser.add_argument('--load_model_path', dest='mod_loc',
        default='./models', type=str)
    parser.add_argument('--load_on_gpu', type=int, dest='gpu_id',
        help='The GPU ID loading model. Use single GPU.')
    
    # Optional for Train, Valid and Infer
    parser.add_argument('--use_lora', action='store_ture', default=False)
    parser.add_argument('--lora_path', type=str, 
        help="Needed by mode valid and infer, please enable use_lora and give the path to LoRA models.")
    
    # mode infer
    parser.add_argument('--dataset', dest='iset', type=str)
    
    # mode compile_model
    parser.add_argument('--model_path', dest='mod2cmp', type=str)
    # see https://tvm.apache.org/docs/reference/api/python/target.html#module-tvm.target for detals.
    parser.add_argument('--compile_mode', dest='cmp_mode', type=str, choices=Target.list_kinds())
    parser.add_argument('--save_package_path', dest='pkg_path', type=str)
    parser.add_argument('--tune_mode', dest='tune', action='store_true', default=False)
    parser.add_argument('--continue_compile', action='store_true', default=False)
    parser.add_argument('--enable_autoscheduler', dest='autoscheduler', action='store_true', default=False)
    parser.add_argument('--set_trails', dest='trails', default=10000, type=int)
    parser.add_argument('--set_timeout', dest='timeout', default=10, type=int)

    # mode show_graphs

    config = ConfigParser()
    args = parser.parse_args()

    if args.f_cfg is None:
        config.read(DEFAULT_CONFIG_PATH)
    else:
        config.read(args.f_cfg)
    
    check_devices()    
    init_dirs()

    match args.mode:
        case 'train':
            train(args, config['Train'], custom_net=args.custom_net, carry_on=args.carry_on)
        case 'valid':
            validate(args=args)
        case 'infer':
            result = inference(args=args)
        case 'compile_model':
            pass
        case 'show_graphs':
            pass
        
def init_dirs():
    FILE_ATTRIBUTE_HIDDEN = 0x02

    if not os.path.exists(args.mod_path):
        os.mkdir(args.mod_path)
    if not os.path.exists(CHECKPOINT_PATH):
        os.mkdir(CHECKPOINT_PATH)
    if not os.path.exists(TENSORBOARD_DATA_PATH):
        os.mkdir(TENSORBOARD_DATA_PATH)
        ret = SetFileAttributesW(
            TENSORBOARD_DATA_PATH, FILE_ATTRIBUTE_HIDDEN)

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