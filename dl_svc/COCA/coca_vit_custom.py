"""
    COCA Vit Custom Model Definition. 
"""
from torch import nn
from dl_svc.COCA.coca_model import coca_vit

def coca_vit_custom():
    """
        Edit it directly if you want to change the struct.
    """
    return coca_vit(
            # Required vision args
            vision_patch_size=32,
            vision_dim_feedforward=3072,
            vision_n_layer=12,
            vision_n_head=12,
            # Required text args
            vocab_size=194,
            num_text_positions=6,
            text_hidden_dim=102,
            text_n_layer=12,
            text_n_head=3,
            text_dim_feedforward=2048,
            text_output_dim=102,
            # Required fusion args
            fusion_n_layer=12,
            fusion_n_head=3,
            fusion_dim_feedforward=2048,
            # Required attention pooler args
            pooler_input_embed_dim=768,
            pooler_output_embed_dim=102,
            pooler_n_head=8,
            # Optional vision args
            image_size=512,
            num_channels=3,
            vision_activation=nn.GELU,
            vision_transformer_dropout=0.0,
            patch_embed_dropout_prob=0.0,
            vision_layer_norm_eps=1e-5,
            vision_final_layer_norm_eps=None,
            vision_norm_first=True,
            vision_include_cls_embed=False,  # This is different from ViT default
            vision_drop_path_rate=None,
            vision_patch_drop_rate=None,
            # Optional text args
            pad_idx=0,
            text_embed_cls=True,
            text_dropout=0.0,
            text_activation=nn.GELU,
            text_layer_norm_eps=1e-5,
            text_norm_first=True,
            text_final_layer_norm_eps=1e-5,
            # Optional fusion args
            fusion_dropout=0.0,
            fusion_activation=nn.GELU,
            fusion_layer_norm_eps=1e-5,
            fusion_norm_first=True,
            fusion_final_layer_norm_eps=1e-5,
            multimodal_output_projection_dim=None, # 49408
            # Optional attention pooler args
            cascaded_pooler=True,
            pooler_n_queries=256,
            pooler_layer_norm_eps=1e-5,
        )
