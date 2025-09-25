# D-Bo

**D-Bo** is a web application enabling users to play a familiar card game together in real-time.

## Table of Contents

- [Overview](#overview)
- [Architecture](#architecture)
- [Features](#features)
- [Roadmap](#roadmap)
- [License](#license)
- [Contact](#contact)

## Overview

This project began with a personal motivation: my mother and her cousin used to enjoy playing a certain card game together regularly. After her cousin moved to Mexico, distance made it impossible to continue. D-Bo is being developed so they — as well as others — can keep playing together online, across any distance.

D-Bo aims to provide an accessible, browser-based environment where players can connect, join lobbies, and play the game in real-time with features that make the experience seamless and secure.

## Architecture

D-Bo follows a client-server architecture:

- **Frontend**

  - Built with **React**, **TypeScript**, **TailwindCSS**, and **Vite**
  - Provides the user interface, lobby management, and game interactions
  - Designed to be responsive and user-friendly

- **Backend**

  - Implemented in **Rust**
  - Exposes a **REST API** using **Axum**
  - Handles **WebSocket** connections for real-time communication between players
  - Manages authentication, session security, and database interactions

- **Database**

  - Uses **MongoDB** to persist user accounts, lobbies, and game state

- **Tooling**
  - Development is currently local-first, but future setup will include **Dockerfiles** and **Docker Compose** for reproducible environments

## Features

- **Authentication**

  - Username + password login
  - Email verification
  - Secure password hashing
  - JWT access tokens (15 min lifetime)
  - Up to 3 persistent refresh tokens stored in database (30 day lifetime)

- **Gameplay**

  - Real-time player-to-player interaction powered by WebSockets

- **Internationalization**

  - English and Spanish translations supported

- **Planned (Low Priority)**
  - Player-to-player messaging in game lobbies
  - Player avatars

## Roadmap

The application is currently in **early development**.  
A detailed six-sprint development plan aiming for a Halloween release can be found in [ROADMAP.md](./ROADMAP.md).

## License

This project is **proprietary**.  
All rights reserved.  
The code in this repository may not be copied, modified, or distributed without explicit permission from the author.

## Contact

- GitHub: [b1gd3vd0g](https://github.com/b1gd3vd0g)
- LinkedIn: [Devin Peevy](https://www.linkedin.com/in/devinpeevy)
- Email: [devin@bigdevdog.com](mailto:devin@bigdevdog.com)
