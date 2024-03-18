# Insect Identifier

## Directory Tree
<details><summary>File Structure</summary>
<pre>
<code>Insect-Identifier
├── Cargo.lock
├── Cargo.toml
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
├── output
├── README.md
└── src
    ├── authenticator.rs
    ├── bin
    │   └── init.rs
    ├── cache.rs
    ├── dataset_io.rs
    ├── feedback.rs
    ├── main.rs
    ├── model_backup.rs
    ├── pic_io.rs
    ├── training_show.rs
    └── user_manager.rs
9 directories, 28 files</code>
</pre>
</details>

## Dependency

### Rust >=v1.56

### PostgreSQL

#### Install & Initialize [Reference](https://blog.csdn.net/Mculover666/article/details/124049857)
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
