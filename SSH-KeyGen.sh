if [ ! $# -lt 1  ];then
  echo param error! need 1 param but got $#.
  exit
fi

dirname=${1:-".key"}
if [ ! -d $dirname  ];then
  mkdir $dirname
fi
/usr/bin/openssl genrsa > "$dirname/cert.key"
/usr/bin/openssl req -new -x509 -key "$dirname/cert.key" > "$dirname/cert.pem"