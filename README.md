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
│   │   ├── inference.py
│   │   ├── prune_model.py
│   │   ├── train.py
│   │   └── validate.py
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

19 directories, 56 files</code>
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

`On dev ......`

### Models' Source [[Reference]](https://github.com/facebookresearch/multimodal)

The deep learning model named COCA, which comes from [TorchMultimodal](https://github.com/facebookresearch/multimodal), is the large model in this project here using for insect image classification. Thanks for their excellent works!

However I extract COCA only and edited source code in order to fit the project in plan of applying `deepspeed`, `Lora Adaptation`, `TVM`, etc.

### Early Stop Source [[Reference]](https://github.com/Bjarten/early-stopping-pytorch)

The project use Early Stop Regularization method to train COCA, because of the lack of data. The Early Stop class comes from `pytorchtools.py` of [here](https://github.com/Bjarten/early-stopping-pytorch).

### Initial params' value [[Reference]](https://arxiv.org/abs/2001.08361)

The deeplearning procedures of project use the initial params' value, which come from the [research](https://arxiv.org/abs/2001.08361). Great appreciation for [Jared Kaplan](https://sites.krieger.jhu.edu/jared-kaplan/)'s research!