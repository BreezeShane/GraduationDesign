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
│   │   ├── coca_vit_custom.py
│   │   ├── multimodal_decoder.py
│   │   └── text_decoder.py
│   ├── config.py
│   ├── DataProcess
│   │   ├── datasetloader.py
│   │   ├── generate_dataset.py
│   │   └── text_processor.py
│   ├── ds_cfg.json
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
├── front_end
│   ├── app
│   │   ├── Componets
│   │   │   ├── Buttons
│   │   │   │   ├── SignInButton.tsx
│   │   │   │   ├── SignOutButton.tsx
│   │   │   │   └── SignUpButton.tsx
│   │   │   ├── FileManage.tsx
│   │   │   ├── NavBar.tsx
│   │   │   ├── ResultPagePanel.tsx
│   │   │   └── UploadImage.tsx
│   │   ├── favicon.ico
│   │   ├── globals.css
│   │   ├── layout.tsx
│   │   ├── page.module.css
│   │   ├── Pages
│   │   │   ├── ContentPanel.tsx
│   │   │   └── SubPages
│   │   │       ├── commands.json
│   │   │       ├── Commands.tsx
│   │   │       ├── Common.tsx
│   │   │       ├── FeedbackManage.tsx
│   │   │       ├── LabelData.tsx
│   │   │       ├── ModelManage.tsx
│   │   │       ├── UserInfo.tsx
│   │   │       ├── UserManage.tsx
│   │   │       └── WebSSH.tsx
│   │   ├── page.tsx
│   │   ├── Types.ts
│   │   └── Utils.tsx
│   ├── next.config.mjs
│   ├── next-env.d.ts
│   ├── package.json
│   ├── package-lock.json
│   ├── public
│   │   ├── next.svg
│   │   └── vercel.svg
│   ├── README.md
│   └── tsconfig.json
├── manager.py
├── README.md
├── requirements.txt
├── src
│   ├── authenticator.rs
│   ├── config.rs
│   ├── daemon.rs
│   ├── dl_svc.rs
│   ├── doc_database.rs
│   ├── feedback.rs
│   ├── init_proj
│   │   └── init.rs
│   ├── io_agent.rs
│   ├── main.rs
│   ├── model_manager.rs
│   ├── ssh_socket
│   │   ├── client.rs
│   │   └── server.rs
│   └── user_manager.rs
├── SSH-KeyGen.sh
└── SSHwifty.yml

20 directories, 79 files</code>
</pre>
</details>

## Dependency

### Next.js + Ant-design

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

### SSH Wifty [[Reference]](https://github.com/nirui/sshwifty) + Docker + Docker-compose

The deeplearning server should deploy up SSH wifty server based on Go-lang.

For the project, you could run the command to install & start docker service:
```shell
docker-compose -f SSHwifty.yml up -d
```
And you could run the command to stop docker service:
```shell
docker-compose -f SSHwifty.yml stop
```
If you would like to stop and remove the docker service, run this:
```shell
docker-compose -f SSHwifty.yml down
```

The expected run result should be like here:
```shell
❯ docker-compose -f SSHwifty.yml up -d
[+] Running 5/5
 ✔ sshwifty Pulled                                                                                                                    1935.4s
   ✔ 4abcf2066143 Already exists                                                                                                         0.0s
   ✔ bdaaca02b8af Pull complete                                                                                                       1175.2s
   ✔ 17dda63926e9 Pull complete                                                                                                       1919.3s
   ✔ 0360c3b1c676 Pull complete                                                                                                       1919.3s
[+] Running 2/2
 ✔ Network proj_default  Created                                                                                                         0.1s
 ✔ Container sshwifty    Started                                                                                                         0.6s
```

If it's need to use RSA to secure the connection to deeplearning server, you could simply run the shell `SSH-KeyGen.sh`:
```shell
sh ./SSH-KeyGen.sh
```

<center><strong>⚠️ To use SSH Wifty, you should ensure that the "sshd" daemon is running. Use <code>systemctl start sshd</code> to start sshd service.</strong></center>

### CUDA == v11.7 (Not ensured to support newer version) **[On dev]**

In general, the deep learning would support higher version, as long as DeepSpeed supports PyTorch and PyTorch supports the relative CUDA.

To install PyTorch v1.13.1+cu117, use the command **(Of course torchaudio is optional)**:
```shell
pip install torch==1.13.1+cu117 torchvision==0.14.1+cu117 torchaudio==0.13.1 --extra-index-url https://download.pytorch.org/whl/cu117
```

The command comes from `pytorch.org`, for more details, see: [INSTALLING PREVIOUS VERSIONS OF PYTORCH - pytorch.org](https://pytorch.org/get-started/previous-versions/)


## Deep Learning

### Using Datasets

<!-- > The HTML code of tables comes from here: [Tables Generator](https://www.tablesgenerator.com/html_tables) -->

[IP102: A Large-Scale Benchmark Dataset for Insect Pest Recognition](https://github.com/xpwu95/IP102?tab=readme-ov-file)

    The IP102 dataset contains more than 75,000 images belongs to 102 categories. [Source](https://drive.google.com/drive/folders/1svFSy2Da3cVMvekBwe13mzyx38XZ9xWo?usp=sharing) The dataset contains 45,095 images in the training set, 7,508 images in the validation set, and 22,619 images in the testing set for classification task.
<!-- 2. [Data Set of 120 Insect Species for Classification projects - kaggle](https://www.kaggle.com/discussions/general/164015)

    It has 291 species of Insects using 63,364 images from the Natural History Museum London. [Source](https://zenodo.org/record/3549369#.XvI_jMfVLIU)
3. [InsectBase: Soybean Crop Insect Raw Image Dataset_V1 with Bounding boxes for Classification and Localization](https://figshare.com/articles/dataset/Soybean_Crop_Insect_Raw_Image_Dataset_V1_with_bounding_boxes/13077221/4)

    The dataset contains 4 catecories: Eocanthecona Bug, Tobacco Caterpillar, Red Hairy Catterpillar, Larva Spodoptera. It's a total of 3824 images.
4. [Insect Village Synthetic Dataset - kaggle](https://www.kaggle.com/datasets/vencerlanz09/insect-village-synthetic-dataset?resource=download-directory&select=Insect+Classes)

    The project use the dataset's folder `Insect Classes`, contains 1000 synthetic images for each insect class(10 categories and 10000 images in total).
5. [Dangerous Farm Insects Dataset - kaggle](https://www.kaggle.com/datasets/tarundalal/dangerous-insects-dataset)

    This dataset contains 15 classes that are regarded as the dangerous and harmful insects(Images total in 1578).
6. [Insect Detect - insect classification dataset v2](https://zenodo.org/records/8325384)

    The dataset contains 27 classes and 21000 images in total.
    <details>
    <summary>Count of each class</summary>
    <style type="text/css">
    .tg  {border-collapse:collapse;border-spacing:0;}
    .tg td{border-color:black;border-style:solid;border-width:1px;font-family:Arial, sans-serif;font-size:14px;
    overflow:hidden;padding:10px 5px;word-break:normal;}
    .tg th{border-color:black;border-style:solid;border-width:1px;font-family:Arial, sans-serif;font-size:14px;
    font-weight:normal;overflow:hidden;padding:10px 5px;word-break:normal;}
    .tg .tg-baqh{text-align:center;vertical-align:top}
    .tg .tg-0lax{text-align:left;vertical-align:top}
    </style>
    <table class="tg">
    <thead>
    <tr>
        <th class="tg-baqh">Class</th>
        <th class="tg-baqh">Description</th>
        <th class="tg-baqh">Image Count</th>
    </tr>
    </thead>
    <tbody>
    <tr>
        <td class="tg-baqh">ant</td>
        <td class="tg-0lax">Formicidae</td>
        <td class="tg-baqh">1097</td>
    </tr>
    <tr>
        <td class="tg-baqh">bee</td>
        <td class="tg-0lax">Anthophila excluding Apis mellifera and Bombus sp.</td>
        <td class="tg-baqh">1061</td>
    </tr>
    <tr>
        <td class="tg-baqh">bee_apis</td>
        <td class="tg-0lax">Apis mellifera</td>
        <td class="tg-baqh">294</td>
    </tr>
    <tr>
        <td class="tg-baqh">bee_bombus</td>
        <td class="tg-0lax">Bombus sp.</td>
        <td class="tg-baqh">1262</td>
    </tr>
    <tr>
        <td class="tg-baqh">beetle</td>
        <td class="tg-0lax">Coleoptera excluding Coccinellidae and some Oedemeridae</td>
        <td class="tg-baqh">520</td>
    </tr>
    <tr>
        <td class="tg-baqh">beetle_cocci</td>
        <td class="tg-0lax">Coccinellidae</td>
        <td class="tg-baqh">776</td>
    </tr>
    <tr>
        <td class="tg-baqh">beetle_oedem</td>
        <td class="tg-0lax">Visually distinct Oedemeridae</td>
        <td class="tg-baqh">199</td>
    </tr>
    <tr>
        <td class="tg-baqh">bug</td>
        <td class="tg-0lax">Heteroptera excluding Graphosoma italicum</td>
        <td class="tg-baqh">390</td>
    </tr>
    <tr>
        <td class="tg-baqh">bug_grapho</td>
        <td class="tg-0lax">Graphosoma italicum</td>
        <td class="tg-baqh">185</td>
    </tr>
    <tr>
        <td class="tg-baqh">fly</td>
        <td class="tg-0lax">Brachycera excluding Empididae, Sarcophagidae, Syrphidae and small Brachycera</td>
        <td class="tg-baqh">1717</td>
    </tr>
    <tr>
        <td class="tg-baqh">fly_empi</td>
        <td class="tg-0lax">Empididae</td>
        <td class="tg-baqh">177</td>
    </tr>
    <tr>
        <td class="tg-baqh">fly_sarco</td>
        <td class="tg-0lax">Visually distinct Sarcophagidae</td>
        <td class="tg-baqh">319</td>
    </tr>
    <tr>
        <td class="tg-baqh">fly_small</td>
        <td class="tg-0lax">Small Brachycera</td>
        <td class="tg-baqh">1662</td>
    </tr>
    <tr>
        <td class="tg-baqh">hfly_episyr</td>
        <td class="tg-0lax">Hoverfly Episyrphus balteatus</td>
        <td class="tg-baqh">2518</td>
    </tr>
    <tr>
        <td class="tg-baqh">hfly_eristal</td>
        <td class="tg-0lax">Hoverfly Eristalis sp., mainly Eristalis tenax</td>
        <td class="tg-baqh">1954</td>
    </tr>
    <tr>
        <td class="tg-baqh">hfly_eupeo</td>
        <td class="tg-0lax">Mainly hoverfly Eupeodes corollae and Scaeva pyrastri</td>
        <td class="tg-baqh">1358</td>
    </tr>
    <tr>
        <td class="tg-baqh">hfly_myathr</td>
        <td class="tg-0lax">Hoverfly Myathropa florea</td>
        <td class="tg-baqh">593</td>
    </tr>
    <tr>
        <td class="tg-baqh">hfly_sphaero</td>
        <td class="tg-0lax">Hoverfly Sphaerophoria sp., mainly Sphaerophoria scripta</td>
        <td class="tg-baqh">374</td>
    </tr>
    <tr>
        <td class="tg-baqh">hfly_syrphus</td>
        <td class="tg-0lax">Mainly hoverfly Syrphus sp.</td>
        <td class="tg-baqh">488</td>
    </tr>
    <tr>
        <td class="tg-baqh">lepi</td>
        <td class="tg-0lax">Lepidoptera</td>
        <td class="tg-baqh">228</td>
    </tr>
    <tr>
        <td class="tg-baqh">none_bg</td>
        <td class="tg-0lax">Images with no insect - background (platform)</td>
        <td class="tg-baqh">851</td>
    </tr>
    <tr>
        <td class="tg-baqh">none_bird</td>
        <td class="tg-0lax">Images with no insect - bird sitting on platform</td>
        <td class="tg-baqh">67</td>
    </tr>
    <tr>
        <td class="tg-baqh">none_dirt</td>
        <td class="tg-0lax">Images with no insect - leaves and other plant material, bird droppings</td>
        <td class="tg-baqh">838</td>
    </tr>
    <tr>
        <td class="tg-baqh">none_shadow</td>
        <td class="tg-0lax">Images with no insect - shadows of insects or surrounding plants</td>
        <td class="tg-baqh">647</td>
    </tr>
    <tr>
        <td class="tg-baqh">other</td>
        <td class="tg-0lax">Other Arthropods, including various Hymenoptera and Symphyta, Diptera, Orthoptera, <br>Auchenorrhyncha, Neuroptera, Araneae</td>
        <td class="tg-baqh">790</td>
    </tr>
    <tr>
        <td class="tg-baqh">scorpionfly</td>
        <td class="tg-0lax">Panorpa sp.</td>
        <td class="tg-baqh">120</td>
    </tr>
    <tr>
        <td class="tg-baqh">wasp</td>
        <td class="tg-0lax">Mainly Vespula sp. and Polistes dominula</td>
        <td class="tg-baqh">515</td>
    </tr>
    </tbody>
    </table>
    </details> -->

### Data Process

#### Generate Dataset for COCA

Because COCA need two input(Images and Text), so it's necessary to generate a new dataset from `Using Datasets` above. The run result is below:

```shell
❯ python dl_svc/DataProcess/generate_dataset.py
100%|██████████████████████████████████████████████████████████████████████████████████| 45095/45095 [00:11<00:00, 3829.90it/s]
100%|████████████████████████████████████████████████████████████████████████████████████| 7508/7508 [00:03<00:00, 2270.78it/s]
100%|██████████████████████████████████████████████████████████████████████████████████| 22619/22619 [00:10<00:00, 2124.11it/s]

```

### Models' Source [[Reference]](https://github.com/facebookresearch/multimodal)

The deep learning model named COCA, which comes from [TorchMultimodal](https://github.com/facebookresearch/multimodal), is the large model in this project here using for insect image classification. Thanks for their excellent works!

However I extract COCA only and edited source code in order to fit the project in plan of applying `deepspeed`, `Lora Adaptation`, `TVM`, etc.

### Early Stop Source [[Reference]](https://github.com/Bjarten/early-stopping-pytorch)

The project use Early Stop Regularization method to train COCA, because of the lack of data. The Early Stop class comes from `pytorchtools.py` of [here](https://github.com/Bjarten/early-stopping-pytorch).

### Initial params' value [[Reference]](https://arxiv.org/abs/2001.08361)

The deeplearning procedures of project use the initial params' value, which come from the [research](https://arxiv.org/abs/2001.08361). Great appreciation for [Jared Kaplan](https://sites.krieger.jhu.edu/jared-kaplan/)'s research!