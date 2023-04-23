#### how to use
 - create .env in base-dir
   - DATABASE_URL=mysql://name:password@server/simple_proxy
   - PORT=8080
 - create .env in client-dir
   - VITE_SERVER_URL=http://localhost:8080
 - run docker compose