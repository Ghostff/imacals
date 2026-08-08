# syntax=docker/dockerfile:1.7

FROM node:20-alpine AS dev
WORKDIR /app

ENV FORCE_COLOR=1
ENV TERM=xterm-256color
ENV COLORTERM=truecolor

COPY . .

EXPOSE 5174

CMD ["sh", "-c", "npm install --no-audit --no-fund && npm run dev"]
