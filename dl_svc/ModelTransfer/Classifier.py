import torch
from torch import nn
from Layers.attention_pooler import AttentionPooler
from Encoder.vision_transformer import vision_transformer

class Classifier(nn.Module):
    def __init__(pretrained_model_path: str, device, model_type="normal"):
        self.model_type = model_type.lower()
        self.device = device
        self.pretrained_model = vision_transformer(
            patch_size=32,
            hidden_dim=768,
            dim_feedforward=3072,
            n_layer=12,
            n_head=12,
            image_size=512,
            num_channels=3,
            activation=nn.GELU,
            transformer_dropout=0.0,
            patch_embed_dropout_prob=0.0,
            layer_norm_eps=1e-5,
            final_layer_norm_eps=None,
            norm_first=True,
            include_cls_embed=False,
            drop_path_rate=None,
            patch_drop_rate=None,
        )
        self.attention_pooler = AttentionPooler(
            input_embed_dim=512,
            output_embed_dim=102,
            n_head=8,
            n_queries=1,
            layer_norm_eps=1e-5,
        )
        self.softmax = nn.Softmax(dim=1)

        assert self.model_type in ["normal", "lora", "deepspeed"]

        match self.model_type:
            case "normal":
                trained_model = torch.load(pretrained_model_path, map_location=self.device)['model']
                model_dict = self.pretrained_model.state_dict()
                load_dict = { k:v for k, v in trained_model.items() if k in model_dict.keys()}
                model_dict.update(load_dict)
                self.pretrained_model.load_state_dict(model_dict)
                self.pretrained_model.eval()
            case "lora":
                pass
            case "deepspeed":
                pass
            case _:
                raise ValueError("Wrong Model Type! Acceptable model types are normal, lora and deepspeed.")

        for name, parameter in self.pretrained_model.named_parameters():
            parameter.requires_grad = False

    def forward(self, x):
        with torch.no_grad():
            self.pretrained_model.eval()
            x = self.pretrained_model(x)
        x = self.attention_pooler(x)
        x = x.squeeze(1)
        x = self.softmax(x)
        return x