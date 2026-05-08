# sugoi-rec
> 🇧🇷 Veja a versão em português [abaixo](#português).

---

## English

**sugoi-rec** is a full-stack portfolio project for tracking anime and receiving personalized recommendations.
Built with FastAPI, PostgreSQL and React, integrated with the [AniList API](https://anilist.co).

### Tech Stack
- **Backend:** FastAPI, PostgreSQL, SQLAlchemy, Alembic, JWT auth
- **Frontend:** React, Vite, Tailwind CSS
- **External API:** AniList GraphQL API

### Features
- ✅ User authentication (register, login, JWT)
- ✅ Anime search and cache (by name or AniList ID)
- ✅ Anime tracking with status (watching, completed, dropped, planned)
- ✅ Ratings and favorites
- ✅ Analytics dashboard (top genres, average rating, status count)
- ✅ Personalized recommendation engine based on taste profile
- ✅ Anime detail page (description, genres, tags)
- ✅ Retro pixel art UI, responsive for mobile and desktop
- ⬜ AniList account sync

### Running locally

**Backend:**
```bash
cd sugoi-rec
python3 -m venv venv
source venv/bin/activate
pip install -r requirements.txt
# create a .env file with DATABASE_URL and JWT_SECRET_KEY
alembic upgrade head
uvicorn app.main:app --reload
```

**Frontend:**
```bash
cd frontend
npm install
npm run dev
```

API docs available at `http://localhost:8000/docs`

---

## Português

**sugoi-rec** é um projeto full-stack de portfólio para registrar animes e receber recomendações personalizadas.
Desenvolvido com FastAPI, PostgreSQL e React, integrado com a [AniList API](https://anilist.co).

### Tecnologias
- **Backend:** FastAPI, PostgreSQL, SQLAlchemy, Alembic, autenticação JWT
- **Frontend:** React, Vite, Tailwind CSS
- **API externa:** AniList GraphQL API

### Funcionalidades
- ✅ Autenticação de usuários (registro, login, JWT)
- ✅ Busca e cache de animes (por nome ou ID do AniList)
- ✅ Registro de animes com status (assistindo, concluído, dropado, planejado)
- ✅ Avaliações e favoritos
- ✅ Dashboard de analytics (top gêneros, média de notas, contagem por status)
- ✅ Motor de recomendações personalizadas baseado no perfil de gosto
- ✅ Página de detalhes do anime (descrição, gêneros, tags)
- ✅ Interface retro pixel art, responsiva para mobile e desktop
- ⬜ Sincronização com conta AniList

### Rodando localmente

**Backend:**
```bash
cd sugoi-rec
python3 -m venv venv
source venv/bin/activate
pip install -r requirements.txt
# crie um arquivo .env com DATABASE_URL e JWT_SECRET_KEY
alembic upgrade head
uvicorn app.main:app --reload
```

**Frontend:**
```bash
cd frontend
npm install
npm run dev
```

Documentação da API disponível em `http://localhost:8000/docs`