from yacs.config import CfgNode as CN

TRAIN_CFG = CN()
# ----------------------------#
TRAIN_CFG.SEED = 42
TRAIN_CFG.BATCH_SIZE = 32
TRAIN_CFG.LR = 0.002123 # 0.003239 + −0.0001395log_2(N) where N means the number of embedding params.
TRAIN_CFG.WARM_UP = True
TRAIN_CFG.EPOCHS = 3
TRAIN_CFG.EARLY_STOP = True
TRAIN_CFG.PATIENCE = 7

