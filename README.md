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
│   ├── Utils
│   │   ├── attention.py
│   │   ├── common.py
│   │   ├── distributed.py
│   │   └── file_io.py
│   └── utils.py
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
├── README.md
├── requirements.txt
└── src
    ├── authenticator.rs
    ├── bin
    │   └── init.rs
    ├── config.rs
    ├── daemon.rs
    ├── dl_svc.rs
    ├── doc_database.rs
    ├── feedback.rs
    ├── io_cache.rs
    ├── main.rs
    ├── model_manager.rs
    └── user_manager.rs

16 directories, 47 files</code>
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

### Models' Source [[Reference]](https://github.com/facebookresearch/multimodal)

The deep learning model named COCA, which comes from [TorchMultimodal](https://github.com/facebookresearch/multimodal), is the large model in this project here using for insect image classification. Thanks for their excellent works!

However I extract COCA only and edited source code in order to fit the project in plan of applying `deepspeed`, `Lora Adaptation`, `TVM`, etc.