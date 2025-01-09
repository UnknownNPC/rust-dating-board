# Rust Dating Board

## Overview

Rust Dating Board is a web application for creating and managing dating profiles. This is my first project in Rust, created to explore programming in the language. Despite being a learning project, it is fully functional.

## Features

- User authentication with JWT and Google OAuth.
- Dynamic profile and photo management.
- Migration system with SeaORM.
- Internationalization support using `rust-i18n`.
- Efficient image processing with `image` and `imageproc`.

---

## Key Frameworks and Libraries

### Backend

- **[Actix-Web](https://actix.rs/)**: A powerful, pragmatic, and extremely fast web framework.
- **Actix-Files**: For serving static files.
- **SeaORM**: An async & dynamic ORM for Rust with a powerful migration tool.

### Database

- **PostgreSQL**: Relational database for managing user data.
- **SeaORM CLI**: Automatically generates entities and handles migrations.

### Authentication and Security

- **jsonwebtoken**: For handling JWT tokens.
- **Google OAuth**: Simplifies user authentication via Google.
- **Google reCAPTCHA**: Protects against bots.

### Utilities

- **Sailfish**: A high-performance templating engine.
- **Image & ImageProc**: For image manipulation.

---

## Installation

### Prerequisites

- Rust (2021 edition)
- PostgreSQL
- `.env` file with the necessary environment variables (see below)

### Setup

1. **Clone the repository**:

   ```sh
   git clone git@github.com:UnknownNPC/rust-dating-board.git
   cd dating-board
   ```

2. **Install dependencies**:

   ```sh
   cargo build
   ```

3. **Configure the `.env` file**:

   ```env
   SITE_PROTOCOL='http'
   SITE_URL='localhost'
   SITE_PORT='8080'

   DATABASE_URL='postgres://user:password@localhost:5432/db'

   JWT_SECRET='your_jwt_secret'
   JWT_MAXAGE=44640

   OAUTH_GOOGLE_CLIENT_ID='your_google_client_id'
   OAUTH_GOOGLE_CLIENT_SECRET='your_google_client_secret'
   OAUTH_GOOGLE_REDIRECT_URL='http://localhost:8080/sign_in/google'

   CAPTCHA_GOOGLE_ID='your_captcha_id'
   CAPTCHA_GOOGLE_SECRET='your_captcha_secret'
   CAPTCHA_GOOGLE_SCORE=0.7

   ALL_PHOTOS_FOLDER_NAME='photos'
   ```

4. **Run the migrations**:

   ```sh
   cd migration
   cargo run -- up -u $DATABASE_URL
   ```

   Or using SeaORM CLI:

   ```sh
   sea-orm-cli migrate up
   ```

5. **Start the application**:

   ```sh
   cargo run
   ```

---

### Build

To build the project in release mode, use the following command:

```sh
cargo build --release
```

---

### Server Setup

It is recommended to set up the application behind an NGINX reverse proxy for better performance and security. Configure NGINX to forward requests to the application running on `localhost:8080`.

