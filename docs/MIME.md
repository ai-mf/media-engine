# Check MIME type detection
file --mime-type test.avid
# Should output: video/avid

# Check MIME info
gvfs-info test.avid | grep "standard::content-type"
# Should show: video/avid

# Check desktop association
xdg-mime query filetype test.avid
# Should show: video/avid

# Test the 'file' command
file test.avid
# Should show: MP4 with AIMF metadata