import torch
from os.path import join, splitext, isfile, basename

from ModelTransfer.Classifier import Classifier

def load_classifier_model(pretrained_model_path, device):
    model = Classifier(pretrained_model_path=pretrained_model_path, device=device, model_type="whole")
    model.eval()
    return model

def transfer_pytorch_model_to_torch_script(model, model_input, model_name, model_state_dict_path, save_dest_dir):
    if not isfile(model_state_dict_path):
        raise ValueError(f"The path to pretrained model is required, but got {model_state_dict_path} which is not file!")
    model_file_name, ext = splitext(basename(model_state_dict_path))
    loaded_dict = torch.load(model_state_dict_path)
    model.load_state_dict(loaded_dict)
    model.eval()
    scripted_model = torch.jit.trace(model, model_input)
    scripted_model.save(join(save_dest_dir, f"{model_file_name}.ts{ext}"))
    print(f"Succeeded to transfer the PyTorch model ({model_name}) to TorchScript model!")
    return scripted_model


def transfer_classifier_model(args):
    if None in (args.mod2cmp, args.save_path):
        raise ValueError("All params '--model_path, --save_path' are required!")
    model = load_classifier_model(pretrained_model_path=args.mod2cmp, device=args.device)

    transfer_pytorch_model_to_torch_script(
        model=model, model_input=torch.rand(1, 3, 512, 512), model_name="Classifier",
        model_state_dict_path=args.mod2cmp, save_dest_dir=args.save_path
    )
