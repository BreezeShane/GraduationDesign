"""
    Global Config and Train Config.
"""
from yacs.config import CfgNode as CN

# CHECKPOINT_PATH = './checkpoint/'
# TENSORBOARD_DATA_PATH = './log/'
CHECKPOINT_PATH = '/hy-tmp/checkpoint/'
TENSORBOARD_DATA_PATH = '../tf_logs/'
COMPILED_MODEL_DIR = "./models/compiled/"
MODEL_TUNING_JSON = "./models/compiled/Classifier-Autotuning.json"
IMG_TYPE_LIST = {'jpg','bmp','png','jpeg','rgb','tif'}

TRAIN_CFG = CN()
# ----------------------------#
TRAIN_CFG.SEED = 42
TRAIN_CFG.BATCH_SIZE = 56
TRAIN_CFG.LR = 1e-5
TRAIN_CFG.WARM_UP = True
TRAIN_CFG.EPOCHS = 1
TRAIN_CFG.LOSS_BALANCE = 0.5
TRAIN_CFG.EARLY_STOP = True
TRAIN_CFG.PATIENCE = 7

TRANSFER_TRAIN_CFG = CN()
# ----------------------------#
TRANSFER_TRAIN_CFG.SEED = 42
TRANSFER_TRAIN_CFG.BATCH_SIZE = 64
TRANSFER_TRAIN_CFG.LR = 1e-5
TRANSFER_TRAIN_CFG.WARM_UP = True
TRANSFER_TRAIN_CFG.EPOCHS = 10
