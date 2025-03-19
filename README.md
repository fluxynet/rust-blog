# Authentication

This is a microservice to handle the authentication process.
Authentication is done using Github login.
It handles the Github flow, store relevant data in keydb.

It checks that the user belongs to a specific organization before allowing access.
It communicates via Kafka to provide user information to other services, given a token.

# Articles API

The API is a RESTful API meant for the admin backoffice.
The entities are:

- Article
- Section

An article can have multiple sections.

The database schema is:

## articles

| Field        | Type      | Description                              |
| ------------ | --------- | ---------------------------------------- |
| id           | UUID      | Unique identifier                        |
| title        | String    | Title of the article                     |
| slug         | String    | URL friendly title                       |
| author       | String    | Author of the article                    |
| created_at   | Timestamp | Date the article was created             |
| updated_at   | Timestamp | Date the article was last updated        |
| published_at | Timestamp | Date the article was published           |
| status       | String    | Status of the article (draft, published) |

## sections

| Field      | Type    | Description                            |
| ---------- | ------- | -------------------------------------- |
| id         | UUID    | Unique identifier                      |
| article_id | UUID    | Article the section belongs to         |
| kind       | String  | Type of section (text, image, code)    |
| content    | String  | Content of the section                 |
| position   | Integer | Position of the section in the article |

## published_articles

| Field        | Type      | Description                       |
| ------------ | --------- | --------------------------------- |
| id           | UUID      | Unique identifier                 |
| article_id   | UUID      | Article the section belongs to    |
| title        | String    | Title of the article              |
| slug         | String    | URL friendly title                |
| author       | String    | Author of the article             |
| created_at   | Timestamp | Date the article was created      |
| updated_at   | Timestamp | Date the article was last updated |
| published_at | Timestamp | Date the article was published    |
| content      | String    | Content of the article            |

The API has the following endpoints:

- GET /articles
- POST /articles
- GET /articles/:id
- DELETE /articles/:id
- PATCH /articles/:id
- GET /articles/:id/sections
- POST /articles/:id/sections
- DELETE /articles/:id/sections/:sectionId
- PUT /articles/:id/sections/:sectionId

Data is read from Postgres database.
Mutations trigger events to Kafka.

# Images API

The API is a RESTful API.
The main entity is:

- Image

- GET /articles/:id/images
- POST /articles/:id/images
- GET /articles/:id/images/:imageId
- DELETE /articles/:id/images/:imageId
- GET /images

# Blogs Service

This is an async service.
It listens to events from Kafka.

Events are supported:

- ArticleSaved: When an article is saved, the data is saved in the database.
- ArticleDeleted: When an article is deleted, the data is deleted from the database.
- ArticlePublished: When an article is published, the blog is regenerated.

# Admin Service

This will be
