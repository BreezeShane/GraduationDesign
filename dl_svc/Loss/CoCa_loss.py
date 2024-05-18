from torch import Tensor
from torch.nn import CrossEntropyLoss

from config import TRAIN_CFG
from Loss.contrastive_loss_with_temperature import ContrastiveLossWithTemperature

class CoCaLoss:
    def __init__():
        self.balance = TRAIN_CFG.LOSS_BALANCE
        self.contrastive_loss = ContrastiveLossWithTemperature(
            logit_scale = math.log(1 / 0.07), # DEFAULT_LOGIT_SCALE
            logit_scale_min = math.log(1.0),
            logit_scale_max = math.log(100.0),
        )
        self.caption_loss = CrossEntropyLoss()

    def __call__(self, outputs: list, texts) -> Tensor:
        assert len(outputs) == 3
        return self.forward(outputs=outputs, texts=texts)

    def forward(self, outputs, texts) -> Tensor:
        # outputs = [
        #     contrastive_image_embeddings,
        #     contrastive_text_embeddings,
        #     multimodal_embeddings,
        # ]
        con_loss = self.contrastive_loss(outputs[0].squeeze(), outputs[1])
        cap_loss = self.caption_loss(outputs[2], texts)
        return self.balance * con_loss + (1 - self.balance) * cap_loss