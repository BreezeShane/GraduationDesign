"""
    Generate Formatted Dataset for fitting CoCa model.
    The code is not used for the project directly. It can be run directly.
"""
from os import mkdir
from tqdm import tqdm
from shutil import copy2 as cp
from os.path import join, exists

class DatasetGenerator:
    """ Get path list of image name with label."""
    def __init__(self, src_dataset_path, dest_dataset_path, dest_sub_folder_name, record_file):
        self.label_data_list = []
        self.label_list = []
        self.src_dataset_path = src_dataset_path
        self.dest_dataset_path = dest_dataset_path
        self.dest_path = join(dest_dataset_path, dest_sub_folder_name)

        if not exists(src_dataset_path):
            raise ValueError(f"Dataset Source is needed, but got {src_dataset_path} doesn't exist!")

        self.__init_dirs()

        images_path = join(src_dataset_path, "images")
        raw_text = None
        with open(join(src_dataset_path, record_file), encoding="utf-8") as f:
            raw_text = f.read().splitlines()
        label_data_list = []
        for raw_line in raw_text:
            temp = raw_line.split(' ')
            label = int(temp[1])
            image_name = temp[0]
            label_data_list.append(
                (label, join(images_path, image_name))
            )
            if label not in self.label_list:
                self.label_list.append(label)
        self.label_data_list = label_data_list

    def generate_dataset(self):
        self.__make_label_dirs()
        for (label, image_path) in tqdm(self.label_data_list):
            __dest_path = join(self.dest_path, str(label))
            cp(image_path, __dest_path)

    def __init_dirs(self):
        """ Make dirs according to label list. """
        if not exists(self.dest_dataset_path):
            mkdir(self.dest_dataset_path)
        if not exists(self.dest_path):
            mkdir(self.dest_path)

    def __make_label_dirs(self):
        for label in self.label_list:
            dir_path = join(self.dest_path, str(label))
            if not exists(dir_path):
                mkdir(dir_path)


    def get_dataset_list(self):
        """ Get datset list """
        return self.label_data_list

    def get_all_labels(self):
        """ Get all labels in dataset. """
        return self.label_list

if __name__ == "__main__":
    SRC_DATASET_PATH = "./datasets/IP102_v1.1/"
    DEST_DATASET_PATH = "./datasets/IP102_for_CoCa"

    train_dataset_list = DatasetGenerator(SRC_DATASET_PATH, DEST_DATASET_PATH, "train", "train.txt")
    valid_dataset_list = DatasetGenerator(SRC_DATASET_PATH, DEST_DATASET_PATH, "valid", "val.txt")
    test_dataset_list = DatasetGenerator(SRC_DATASET_PATH, DEST_DATASET_PATH, "test", "test.txt")

    train_dataset_list.generate_dataset()
    valid_dataset_list.generate_dataset()
    test_dataset_list.generate_dataset()
