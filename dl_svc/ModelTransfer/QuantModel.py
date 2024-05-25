import torch
from torch import nn
from tqdm import tqdm
from os.path import join
from torch.quantization import QuantStub, DeQuantStub, get_default_qconfig, prepare, convert

class QuantModel(nn.Module):
    def __init__(self, model, target="x86") -> None:
        assert target.lower() in ["x86", "arm"]

        super().__init__()
        self.quant = QuantStub()
        self.model = model
        self.model.eval()
        self.dequant = DeQuantStub()

        self.qconfig = get_default_qconfig('fbgemm' if target.lower() == "x86" else "qnnpack")
    
    def forward(self, x):
        x = self.quant(x)
        x = self.model(x)
        x = self.dequant(x)
        return x

def prepare_model(model_to_quantize):
    return prepare(model_to_quantize)

def save_qmodel(quantized_model, save_path):
    torch.save(quantized_model.state_dict(), join(save_path, "quantized-model.pth"))
    scripted_model = torch.jit.trace(quantized_model, torch.rand(1, 3, 512, 512)).eval()
    torch.jit.save(scripted_model, join(save_path, "quantized-model.ts.pt"))

def eval_data_distribution(quantized_model, dataloader):
    for _, image in tqdm(dataloader, desc="Evaluating Dataset Distribution: "):
        quantized_model(image)

def quant_model(prepared_model):
    return convert(prepared_model)