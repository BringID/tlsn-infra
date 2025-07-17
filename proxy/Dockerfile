FROM python:3.13.5-slim-bookworm

WORKDIR /root

COPY requirements.txt requirements.txt
COPY config /app/websockify_config

RUN pip install --no-cache-dir -r requirements.txt

# Standard ports
EXPOSE 443
EXPOSE 80

CMD ["python", "-m", "websockify", "80", "--target-config", "/app/websockify_config"]