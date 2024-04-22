import os
import imghdr
from torch.utils.data import IterableDataset
import torchvision
from PIL import Image
from torchvision import transforms
from torch.utils.data import DataLoader, Dataset

IMG_TYPE_LIST = {'jpg','bmp','png','jpeg','rgb','tif'}

class IP102_Dataset(Dataset):
    def __init__(self, dataset_path, data_label_file):
        self.transforms = transforms.Compose([
            transforms.Resize((512,512)),        # Crop to (512,512) size.
            transforms.RandomRotation((30,150)), # Rotate 30°-150° randomly.
            transforms.RandomHorizontalFlip(0.6),# Flip horizontally with 0.6 probability.
            transforms.RandomVerticalFlip(0.4),  # Flip vertically with 0.4 probability.
            transforms.ToTensor(),               # Convert into tensor, and normalize into [0-1], 
                                                 # then convert [W,H,C] to [C,W,H] which PyTorch needs.
            transforms.Normalize(
                mean=(0.5, 0.5, 0.5),
                std=(0.5, 0.5, 0.5))             # Convert [0-1] into [-1, 1].
        ])
        
        if not os.path.isdir(dataset_path):
            raise ValueError(f"Expected the path to dataset, but got {dataset_path}")
        data_label_path = os.path.join(data_label_path, data_label_file)
        images_path = os.path.join(dataset_path, "images")
        try:
            raw_image_label_lines = None
            with open(join(dataset_path, data_label_file)) as f:
                raw_image_label_lines = f.read().splitlines()
            label_data_list = []
            for raw_line in raw_image_label_lines:
                temp = raw_line.split(' ')
                label_data_list.append(
                    (int(temp[1]), join(images_path, temp[0]))
                )
            self.label_data = label_data_list
        except IOError:
            raise IOError("Cannot load dataset! Please check the correct path and keep the file struct of dataset right.")

    def __getitem__(self, index):
        data_with_label = self.label_data[index]
        label = data_with_label[0]
        data = self.transforms(
            Image.open(data_with_label[1])
        )
        return label, data

    def __len__(self):
        assert len(self.data) == len(self.label)
        return len(self.data)


def load_dataset(dataset_folder_path, record_file, batch_size=None) -> DataLoader:
    ip102_dataset = IP102_Dataset(dataset_folder_path, record_file)
    dataloader = DataLoader(dataset=ip102_dataset, batch_size=batch_size, shuffle=True)
    return dataloader


def load_data(data_folder_path):
    data_list = []
    if not os.path.isdir(data_folder_path):
        raise ValueError(f"Data folder should be path, but got {data_folder_path}.")
    for root , _, files in os.walk(data_folder_path): # where _ means middle dirs
        for img_file in files:
            file_path = os.path.join(root, img_file)
            file_type = imghdr.what(file_path)
            if file_type not in imgType_list:
                raise TypeError(f"Needed image type, but got {file_type}")
            data_list.append(
                (file_path, Image.open(file_path))
            )
    return data_list
            