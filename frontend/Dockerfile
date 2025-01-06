FROM node:latest AS build

WORKDIR /app

COPY . .

RUN npm install && npm run build

FROM denoland/deno:latest

WORKDIR /app

COPY --from=build /app .

CMD ["deno", "run", "--allow-env", "--allow-net", "--allow-read", "build/index.js"]
