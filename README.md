# Authentication

This is a microservice to handle the authentication process.
Authentication is done using Github login.
It handles the Github flow, store relevant data in keydb.

## API

| Method | Path                     | Description                                        |
| ------ | ------------------------ | -------------------------------------------------- |
| GET    | /api/auth/login          | Starts the login flow, redirects to github         |
| GET    | /api/auth/login/callback | Callback from github, creates a session and cookie |
| GET    | /api/auth/logout         | Deletes the cookie and session                     |
| GET    | /api/auth/me             | Returns the user data based on cookie              |

# Articles

The API is a RESTful API meant for the admin backoffice.
The only entity is `article`

- Article

An article can have multiple sections.

The database schema is:

## articles

The schema is as follows:

| Field       | Type        | Description                                     |
| ----------- | ----------- | ----------------------------------------------- |
| id          | UUID        | Primary key                                     |
| title       | TEXT        | Title of the article                            |
| description | TEXT        | Description of the article                      |
| content     | TEXT        | Content of the article                          |
| updated_at  | TIMESTAMPTZ | Timestamp of the last update                    |
| created_at  | TIMESTAMPTZ | Timestamp of creation                           |
| status      | TEXT        | Status of the article (published, draft, trash) |
| author      | TEXT        | Author of the article                           |

The API has the following endpoints:

| Method | Path                              | Description                |
| ------ | --------------------------------- | -------------------------- |
| POST   | /api/articles                     | Create a new article       |
| GET    | /api/articles                     | List articles              |
| GET    | /api/articles/{id}                | Get a specific article     |
| PATCH  | /api/articles/{id}                | Update article content     |
| PUT    | /api/articles/{id}/status/publish | Publish article            |
| PUT    | /api/articles/{id}/status/trash   | Move article to trash      |
| PUT    | /api/articles/{id}/status/draft   | Set article to draft       |
| DELETE | /api/articles/{id}                | Permanently delete article |

# Code Structure

The application is a mono-repo and can provide multiple services.

The code is structured as follows:

| Module | Description     |
| ------ | --------------- |
| auth   | authentication  |
| blog   | blog articles   |
| errors | error handling  |
| logs   | logging         |
| web    | common web code |

Sub packages exist to implement traits as needed.

# Running

The application can be executed using the following command:

```sh
./blog [subcommand]
```

where `subcommand` is one of the following:

- auth: starts the authentication service
- admin: starts the admin service
- open-api: generates openapi documentation

The application requires a configuration file: `config.toml`, an example is provided in the repository.
The file need to be in the current working directory.

A dockerfile is provide to build the application as a container.
A docker-compose file is provided to start the application with necessary services.

# Frontend

The frontend is a simple React application that uses the API to manage articles.
For convenience, the frontend code is included in this repository as `apps/admin`.

It uses react-router and shadcn component library.

The frontend dockerfile is provided in `apps/admin/Dockerfile`.
It is a multi-stage build that builds the frontend and serves it using nginx.

