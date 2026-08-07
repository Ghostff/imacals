# syntax=docker/dockerfile:1.7

FROM node:20-alpine AS dev
WORKDIR /app

ENV FORCE_COLOR=1
ENV TERM=xterm-256color
ENV COLORTERM=truecolor

COPY . .

EXPOSE 5174

# @fortawesome/pro-solid-svg-icons lives on FontAwesome's private registry (see .npmrc). It is an
# optionalDependency, so npm skips it when no token is present and the app still boots — the map
# toolbar just falls back to text labels. Warn so the reason is obvious rather than mysterious.
CMD ["sh", "-c", "\
  if [ -z \"$FONTAWESOME_PACKAGE_TOKEN\" ]; then \
    echo ''; \
    echo '──────────────────────────────────────────────────────────────'; \
    echo ' FONTAWESOME_PACKAGE_TOKEN is not set.'; \
    echo ' Skipping @fortawesome/pro-solid-svg-icons (private registry).'; \
    echo ' The app runs; the map toolbar shows text labels instead of icons.'; \
    echo ' To get the icons: add the token to imacals-dashboard/.env'; \
    echo '──────────────────────────────────────────────────────────────'; \
    echo ''; \
  fi; \
  npm install --no-audit --no-fund || npm install --no-audit --no-fund --no-optional; \
  npm run dev"]
