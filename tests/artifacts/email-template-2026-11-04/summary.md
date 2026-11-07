# Password-reset email preview — 2026-11-04

Isolated Askama 0.16.1 compilation and rendering pass with a synthetic reset URL.
Dynamic values remain escaped, and both rendered links preserve the original URL.
Chromium previews pass at 1440px and 390px without horizontal overflow.
The [mobile preview](390.png) is visually inspected. [Desktop preview](1440.png).
No email is sent. This is browser verification, not Gmail/Outlook deliverability or rendering verification.
The existing sender function still needs syntax and error-handling fixes; no full application or Docker build runs.
