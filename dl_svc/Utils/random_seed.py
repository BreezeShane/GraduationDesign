import numpy as np
import random
import torch

def setup_seed(seed):
    """ Set Seed for Replicability. """
    torch.manual_seed(seed)
    torch.cuda.manual_seed_all(seed)
    np.random.seed(seed)
    random.seed(seed)
    torch.backends.cudnn.deterministic = True