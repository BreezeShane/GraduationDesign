import os
from torch.utils.data import IterableDataset
import torchvision
import torchvision.transforms as transforms
from torch.utils.data import DataLoader

class ImageDataset(IterableDataset):
    def __init__(self, filepath):
        fnames = [filepath + '/' + filename for filename in os.listdir(filepath)]
        self.i = -1
        self.compose = [
            transforms.ToPILImage(),
            transforms.Resize((64, 64)),
            transforms.ToTensor(),
            transforms.Normalize(mean=(0.5, 0.5, 0.5), std=(0.5, 0.5, 0.5)),
        ]
    
    def __len__(self):
        return len(fnames)
    
    def __iter__(self):
        return self
    
    def __next__(self):
        self.i += 1
        if self.i >= len(self.fnames):
            raise StopIteration
        img = torchvision.io.read_image(fnames[self.i])
        transform = transforms.Compose(self.compose)
        return transform(img)


def load_dataset(dataset_folder_path, batch_size=None) -> DataLoader:
    iterable_dataset = ImageDataset(dataset_folder_path)
    if batch_size is not None:
        dataloader = DataLoader(dataset=iterable_dataset, batch_size=batch_size)
    else:
        dataloader = DataLoader(dataset=iterable_dataset)
    return dataloader