# Setup Instructions

## Install Plantuml
This is used to generate uml diagrams.
- Install plantuml extension in VScode
- Install plantuml (graphviz, Java JRE, Plantuml jar file and launcher script)
- Then you can press Alt-D in VScode if you are inside a plantuml block
and it will preview the associated diagram.

```
#!/bin/sh -e

# Install Graphvix, Java JRE, Plantuml

sudo apt install -y graphviz default-jre

sudo mkdir -p /opt/plantuml
cd /opt/plantuml

# Download Platuml from official site (See https://plantuml.com/download, it points to sourceforge)

UML=http://sourceforge.net/projects/plantuml/files/plantuml.jar/download
sudo curl -JLO ${UML}

# Generate shell script to launch plantuml
# This requires sudo to create the script in /usr/local/bin/plantuml

cat <<EOF | sudo tee /usr/local/bin/plantuml
#!/bin/sh

java -jar /opt/plantuml/plantuml.jar "\$@"
EOF
sudo chmod a+x /usr/local/bin/plantuml
```
