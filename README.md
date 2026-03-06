```bash
docker run -d \
  --name sing-box-orchestrator \
  --env-file .env \
  -p 8080:8080 \
  -v $(pwd)/sing-box-orchestrator.db:/app/sing-box-orchestrator.db \
  sing-box-config-bot
```