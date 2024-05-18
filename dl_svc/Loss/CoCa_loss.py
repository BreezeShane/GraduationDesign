import math
from torch import Tensor
from torch import nn

from config import TRAIN_CFG
from Loss.contrastive_loss_with_temperature import ContrastiveLossWithTemperature

class CoCaLoss:
    def __init__(self):
        self.balance = TRAIN_CFG.LOSS_BALANCE
        self.contrastive_loss = ContrastiveLossWithTemperature(
            logit_scale = math.log(1 / 0.07), # DEFAULT_LOGIT_SCALE
            logit_scale_min = math.log(1.0),
            logit_scale_max = math.log(100.0),
        )
        self.caption_loss = nn.CrossEntropyLoss()

    def __call__(self, outputs: list, texts) -> Tensor:
        assert len(outputs) == 3
        return self.forward(outputs=outputs, texts=texts)

    def forward(self, outputs, texts) -> Tensor:
        # outputs = [
        #     contrastive_image_embeddings, # (batch_size, n_queries, output_embed_dim)
        #     contrastive_text_embeddings, # (batch_size, output_embed_dim)
        #     multimodal_embeddings, # (batch_size, text_seq_length, output_dim)
        # ]
        con_loss = self.contrastive_loss(outputs[0].squeeze(1), outputs[1])
        cap_loss = self.caption_loss(outputs[2], texts)
        return self.balance * con_loss + (1 - self.balance) * cap_loss