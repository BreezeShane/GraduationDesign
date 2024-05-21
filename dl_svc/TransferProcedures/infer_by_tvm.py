import os
import tvm
from PIL import Image
import numpy as np
from torchvision import transforms
from tvm.contrib import graph_executor

from config import COMPILED_MODEL_DIR

def transform_compose(image_size: int):
    """ Define Image Transform Compose """
    return transforms.Compose([
        transforms.Resize((image_size, image_size)),
        transforms.ToTensor(),

        transforms.Normalize(
            mean=(0.5, 0.5, 0.5),
            std=(0.5, 0.5, 0.5)
        )
    ])

def init_graph_executor(prefix_name, target):
    """ Initialize graph exectutor and then return to Rust. """
    dev = tvm.device(str(target), 0)
    path_lib = os.path.join(COMPILED_MODEL_DIR, f"{prefix_name}_deploy_lib.tar")
    loaded_lib = tvm.runtime.load_module(path_lib)
    module = graph_executor.GraphModule(loaded_lib["default"](dev))
    return module

def run_infer(image_path, module):
    """ Accept module given by Rust and run infer. """
    output_shape = (1, 1, 102)
    compose = transform_compose(image_size=512)
    img = Image.open(image_path).convert("RGB")
    input_data = compose(img).unsqueeze(0)
    module.set_input("input_img", input_data)
    module.run()
    tvm_output = module.get_output(0, tvm.nd.empty(output_shape)).numpy()

    label = np.argmax(tvm_output)
    return label

if __name__ == "__main__":
    import sys

    prefix_name = sys.argv[1]
    target = sys.argv[2]
    image_path = sys.argv[3]
    module = init_graph_executor(prefix_name=prefix_name, target=target)
    result = run_infer(image_path=image_path, module=module)
    print(result)
