"""
    IP102 Dataloader definition.
"""
import os
import imghdr
from os.path import join
from PIL import Image
import torch
from torchvision import transforms
from torch.utils.data import DataLoader, Dataset
from dl_svc.config import IMG_TYPE_LIST
from dl_svc.DataProcess.text_processor import text_process, Converter


class IP102Dataset(Dataset):
    """ The Dataset class used for IP102. """
    def __init__(self, dataset_path, data_label_file, converter: Converter, image_size: int=224):
        self.transforms = transforms.Compose([
            transforms.Resize((image_size, image_size)),    # Crop to (512,512) size.
            transforms.RandomRotation((30,150)),            # Rotate 30°-150° randomly.
            transforms.RandomHorizontalFlip(0.6),           # Flip horizontally with 0.6 probability.
            transforms.RandomVerticalFlip(0.4),             # Flip vertically with 0.4 probability.
            transforms.ToTensor(),                          # Convert into tensor, and normalize into [0-1],
                                                            # then convert [W,H,C] to [C,W,H] (PyTorch needs).
            transforms.Normalize(
                mean=(0.5, 0.5, 0.5),
                std=(0.5, 0.5, 0.5))             # Convert [0-1] into [-1, 1].
        ])
        self.converter = converter

        if not os.path.isdir(dataset_path):
            raise ValueError(f"Expected the path to dataset, but got {dataset_path}")
        data_label_path = os.path.join(dataset_path, data_label_file)
        images_path = os.path.join(dataset_path, "images")
        try:
            raw_image_label_lines = None
            with open(data_label_path, encoding="utf-8") as f:
                raw_image_label_lines = f.read().splitlines()
            label_data_list = []
            for raw_line in raw_image_label_lines:
                temp = raw_line.split(' ')
                label_data_list.append(
                    (int(temp[1]), join(images_path, temp[0]))
                )
            self.label_data = label_data_list
        except IOError as exc:
            raise IOError(
                    "Cannot load dataset! Please check the correct path "
                    "and keep the file struct of dataset right."
                ) from exc

    def __getitem__(self, index):
        data_with_label = self.label_data[index]
        label = data_with_label[0]
        label_tensor = torch.tensor(
                self.converter.get_word_vec(label),
                dtype=torch.long
            )
        image_tensor = self.transforms(
            Image.open(data_with_label[1]).convert('RGB')
        )
        return label_tensor, image_tensor

    def __len__(self):
        return len(self.label_data)


def load_dataset(dataset_folder_path, record_file, class_file, shuffle=True, batch_size=1) -> DataLoader:
    """ load IP102 dataset by text file. """
    vocabulary, converter = text_process(dataset_folder_path, class_file, vec_dim=4)
    ip102_dataset = IP102Dataset(dataset_folder_path, record_file, converter, image_size=512)
    _dataloader = DataLoader(dataset=ip102_dataset, batch_size=batch_size, shuffle=shuffle)
    return _dataloader


def load_data(data_folder_path):
    """ Load images under the designated directory. """
    data_list = []
    if not os.path.isdir(data_folder_path):
        raise ValueError(f"Data folder should be path, but got {data_folder_path}.")
    for root , _, files in os.walk(data_folder_path): # where _ means middle dirs
        for img_file in files:
            file_path = os.path.join(root, img_file)
            file_type = imghdr.what(file_path)
            if file_type not in IMG_TYPE_LIST:
                raise TypeError(f"Needed image type, but got {file_type}")
            data_list.append(
                (file_path, Image.open(file_path))
            )
    return data_list
