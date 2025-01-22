# magick convert pieces_sm_big.png markings.png -composite pieces3d_texture.png
magick convert pieces_sm_big.png pieces3d_texture.png
magick convert -background white pieces3d_texture.png -alpha remove -alpha off -blur 0x5 -level 0%,40% pieces_blurred.png
