# Insect Identifier

## Directory Tree
<details><summary>File Structure</summary>
<pre>
<code>Insect-Identifier
├── Cargo.lock
├── Cargo.toml
├── dl_svc
│   ├── COCA
│   │   ├── coca_model.py
│   │   ├── multimodal_decoder.py
│   │   └── text_decoder.py
│   ├── datasetloader.py
│   ├── default
│   │   └── cfg.ini
│   ├── Encoder
│   │   └── vision_transformer.py
│   ├── Layers
│   │   ├── attention_pooler.py
│   │   ├── mlp.py
│   │   ├── multi_head_attention.py
│   │   ├── normalizations.py
│   │   ├── patch_embedding.py
│   │   └── transformer.py
│   ├── Loss
│   │   └── contrastive_loss_with_temperature.py
│   ├── Masking
│   │   └── random_masking.py
│   ├── procedures
│   │   ├── compile_model.py
│   │   ├── infer_et_valid.py
│   │   ├── prune_model.py
│   │   └── train.py
│   └── Utils
│       ├── attention.py
│       ├── common.py
│       ├── distributed.py
│       ├── early_stop.py
│       └── file_io.py
├── frontend_nextjs
│   ├── next.config.mjs
│   ├── next-env.d.ts
│   ├── package.json
│   ├── package-lock.json
│   ├── postcss.config.js
│   ├── public
│   │   ├── next.svg
│   │   └── vercel.svg
│   ├── README.md
│   ├── src
│   │   ├── app
│   │   │   ├── api
│   │   │   ├── favicon.ico
│   │   │   ├── globals.css
│   │   │   ├── layout.tsx
│   │   │   └── page.tsx
│   │   └── pages
│   ├── tailwind.config.ts
│   └── tsconfig.json
├── manager.py
├── README.md
├── requirements.txt
└── src
    ├── authenticator.rs
    ├── config.rs
    ├── daemon.rs
    ├── dl_svc.rs
    ├── doc_database.rs
    ├── feedback.rs
    ├── init_proj
    │   └── init.rs
    ├── io_cache.rs
    ├── main.rs
    ├── model_manager.rs
    ├── ssh_socket
    │   ├── client.rs
    │   └── server.rs
    └── user_manager.rs
<br>
19 directories, 55 files</code>
</pre>
</details>

## Dependency

### Rust >=v1.56

### PostgreSQL

#### Install & Initialize [[Reference]](https://blog.csdn.net/Mculover666/article/details/124049857)
```shell
yay -S postgresql --noconfirm
sudo passwd postgres
sudo su - postgres -c "initdb --locale en_US.UTF-8 -E UTF8 -D '/var/lib/postgres/data'"
systemctl enable postgresql.service
systemctl start postgresql.service
sudo su postgres
createdb InsectSys
cargo run --bin init # Initialize database.
```

## Deep Learning

### Using Datasets

1. [IP102: A Large-Scale Benchmark Dataset for Insect Pest Recognition](https://github.com/xpwu95/IP102?tab=readme-ov-file)

    The IP102 dataset contains more than 75,000 images belongs to 102 categories. [Source](https://drive.google.com/drive/folders/1svFSy2Da3cVMvekBwe13mzyx38XZ9xWo?usp=sharing) The dataset contains 45,095 images in the training set, 7,508 images in the validation set, and 22,619 images in the testing set for classification task.
2. [Data Set of 120 Insect Species for Classification projects - kaggle](https://www.kaggle.com/discussions/general/164015)
    
    It has 291 species of Insects using 63,364 images from the Natural History Museum London. [Source](https://zenodo.org/record/3549369#.XvI_jMfVLIU)
3. [InsectBase: Soybean Crop Insect Raw Image Dataset_V1 with Bounding boxes for Classification and Localization](https://figshare.com/articles/dataset/Soybean_Crop_Insect_Raw_Image_Dataset_V1_with_bounding_boxes/13077221/4)
    
    The dataset contains 4 catecories: Eocanthecona Bug, Tobacco Caterpillar, Red Hairy Catterpillar, Larva Spodoptera. It's a total of 3824 images.
4. [Insect Village Synthetic Dataset - kaggle](https://www.kaggle.com/datasets/vencerlanz09/insect-village-synthetic-dataset?resource=download-directory&select=Insect+Classes)
    
    The project use the dataset's folder `Insect Classes`, contains 1000 synthetic images for each insect class(10 categories and 10000 images in total).
5. [Dangerous Farm Insects Dataset - kaggle](https://www.kaggle.com/datasets/tarundalal/dangerous-insects-dataset)
    
    This dataset contains 15 classes that are regarded as the dangerous and harmful insects(Images total in 1578).

### Models' Source [[Reference]](https://github.com/facebookresearch/multimodal)

The deep learning model named COCA, which comes from [TorchMultimodal](https://github.com/facebookresearch/multimodal), is the large model in this project here using for insect image classification. Thanks for their excellent works!

However I extract COCA only and edited source code in order to fit the project in plan of applying `deepspeed`, `Lora Adaptation`, `TVM`, etc.

### Early Stop Source [[Reference]](https://github.com/Bjarten/early-stopping-pytorch)

The project use Early Stop Regularization method to train COCA, because of the lack of data. The Early Stop class comes from `pytorchtools.py` of [here](https://github.com/Bjarten/early-stopping-pytorch).

### Initial params' value [[Reference]](https://arxiv.org/abs/2001.08361)

The deeplearning procedures of project use the initial params' value, which come from the [research](https://arxiv.org/abs/2001.08361). Great appreciation for [Jared Kaplan](https://sites.krieger.jhu.edu/jared-kaplan/)'s research!