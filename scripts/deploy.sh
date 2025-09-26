#!/bin/zsh

ssh searxing@hackclub.app << EOF
    cd searxing-hc
    git pull
    
    cp scripts/searxing.service ~/.config/systemd/user/searxing.service

    systemctl --user daemon-reload
    systemctl --user enable --now searxing.service
EOF
