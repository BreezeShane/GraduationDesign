# Insect Identifier

## Directory Tree
<details><summary>File Structure</summary>
<pre>
<code>Insect-Identifier
├── Cargo.lock
├── Cargo.toml
├── deep_learning_service
│   ├── deep_learning_service
│   │   ├── asgi.py
│   │   ├── __init__.py
│   │   ├── settings.py
│   │   ├── urls.py
│   │   └── wsgi.py
│   ├── dl_svc
│   │   ├── apis.py
│   │   ├── datasetloader.py
│   │   ├── __init__.py
│   │   ├── network.py
│   │   ├── urls.py
│   │   └── utils.py
│   └── manage.py
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
└── src
    ├── authenticator.rs
    ├── bin
    │   └── init.rs
    ├── config.rs
    ├── daemon.rs
    ├── doc_database.rs
    ├── feedback.rs
    ├── io_cache.rs
    ├── main.rs
    ├── model_manager.rs
    └── user_manager.rs

12 directories, 39 files</code>
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
