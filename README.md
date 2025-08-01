
# API Rust - Weenav

[![Rust](https://img.shields.io/badge/language-rust-orange.svg)](https://www.rust-lang.org/)  
[![C++17](https://img.shields.io/badge/language-c++17-blue.svg)](https://isocpp.org/std/the-standard)  

Ce projet fournit une API écrite en Rust qui utilise un module C++ (basé sur la bibliothèque dbcppp) pour parser des fichiers DBC (CAN database files). L’objectif est de traiter et d’analyser des données CAN à partir de fichiers DBC, avec une interaction vers une base de données MySQL.

---

## Structure du projet
 

```
api_rust/
├── src/
│   ├── decryptCpp/           # Code C++ de parsing DBC (dbcppp + main.cpp)
│   ├── main.rs               # Entrée principale de l’API Rust
│   └── ...
├── Cargo.toml                # Configuration Rust
├── README.md
└── ...
```

- `src/decryptCpp/` contient le code C++ qui utilise la bibliothèque `dbcppp` pour parser les fichiers `.dbc`.
- Le binaire Rust (`api_rust`) fait appel au code C++ compilé pour récupérer les données et les manipuler.
- La base de données MySQL est utilisée pour stocker et exploiter les données CAN traitées.

---

## Prérequis

- Rust (version stable recommandée, ex: 1.70+)
- g++ (>= 9.4, avec support C++17)
- CMake (pour compiler la bibliothèque dbcppp)
- Serveur MySQL/MariaDB en fonctionnement
- Bibliothèques de développement libxml++ (ex: `libxml++-2.6-dev` sur Ubuntu)

---

## Installation & Compilation

### Installe submodule ( dbcppp )

```bash
git submodule update --init --recursive
```

### Compilation de la bibliothèque C++ (dbcppp)

```bash
cd src/decryptCpp/dbcppp/build
rm -rf *
cmake ..
make
```

Cela génère `libdbcppp.so` dans `src/decryptCpp/dbcppp/build`.

### Compilation du code C++ utilisateur

Depuis la racine du projet :

( retourne à la racine du projet )

```bash
g++ -std=c++17 src/decryptCpp/main.cpp \
    -I./src/decryptCpp/dbcppp/include \
    -L./src/decryptCpp/dbcppp/build \
    -ldbcppp -lxml++-2.6 \
    -o src/decryptCpp/main
```

### Compilation du projet Rust

( A la racine du projet )

```bash
cargo build
```

---

## Configuration

Avant de lancer l’application Rust, assurez-vous que :

- Le serveur MySQL est démarré et accessible (ex: `localhost:3306`).
- La base de données et les tables nécessaires sont créées.
- Le fichier DBC est accessible dans le dossier `src/decryptCpp/` (ex: `WEENAV.dbc`).

### modif fichier build.rs

```bash
python --version
```

dans le fichier build.rs mettre la version exacte de python à cette ligne

Il faut prendre que les 2 premier chiffres, exemple il renvoie 3.12.10, il faut seulement 3.12

println!("cargo:rustc-link-lib=python3.12");

et apres il faut installer la version 'dev'
```bash
sudo apt-get update
sudo apt-get install python<version de python>-dev
# exemple sudo apt-get install python3.12-dev
```

Il faut ensuite ajouter le fichier .env

---

## Lancer l’application

( A la racine du projet )

```bash
cargo run
```

En cas d’erreur de connexion MySQL, vérifiez que le serveur est actif et que les identifiants dans la configuration Rust sont corrects.

---