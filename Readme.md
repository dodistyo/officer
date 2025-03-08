# Officer API

**Officer** is a backend service written in Rust, designed to perform daily operations for your infrastructure. It provides a set of API endpoints to interact with your Kubernetes cluster and database, automating common tasks such as retrieving pod details, restarting deployments, and seeding databases.

## Features

- **Get Pods and Image Versions**: Retrieve a list of running pods along with their associated image versions.
- **Restart Deployment**: Restart a Kubernetes deployment to apply new changes.
- **Seed Database**: Populate the database with initial data for application setup.
- **Health Check**: Check the status of the service to ensure it is running properly.

## API Documentation

The API is automatically generated using the **Paperclip** library. You can access the OpenAPI documentation at:

```
GET /api/docs
```

This provides an interactive interface to explore and test the available endpoints.

## Installation & Setup

1. Clone the repository:
   ```sh
   git clone https://github.com/your-repo/officer.git
   cd officer
   ```
2. Install dependencies:
   ```sh
   cargo build
   ```
3. Configure environment variables in `.env` file:
   ```env
    RUST_LOG=info
    RUST_BACKTRACE=1
    API_KEY=""
    OFFICER_SECRET_KEY=""
    OAUTH2_CLIENT_ID=""
    OAUTH2_CLIENT_SECRET=""
    OAUTH2_URL="https://login.microsoftonline.com/<tenant-id>/oauth2/v2.0"
    OAUTH2_USER_INFO_ENDPOINT="https://graph.microsoft.com/oidc/userinfo"
    OAUTH2_JWKS_URL="https://login.microsoftonline.com/<tenant-id>/discovery/v2.0/keys"
    OAUTH2_REDIRECT_URL="http://localhost:8000/auth/oidc/callback"
    USERS="dodiprasetyo@mail.com,dzaka.eryan@mail.com"
   ```
4. Start the service:
   ```sh
   cargo run
   ```
   With auto reload:
   ```sh
    cargo watch -x run
   ```

## Container

- **Docker**:
  ```sh
  docker build -t officer .
  docker run -p 8000:8000 officer
  ```

## License

MIT License

## Contributing

Feel free to submit issues or pull requests to improve the service!

---

Let me know if you need further customization or additional sections!

