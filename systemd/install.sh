#!/bin/sh

sudo ln --force ./*.service ./*.timer /etc/systemd/system
sudo systemctl daemon-reload
sudo systemctl enable --now alacritty_master.timer
