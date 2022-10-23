start: ## Simple script to start all capability providers with links for this demo
	wash ctl link put MC4KXRTFQJHGGH3WX3OCDZZEHAIGSUDZOG533JHJTVN44H3KPBUO4NRW VADNMSIML2XGO2X4TPIONTIC55R2UUQGPPDZPAVSC2QD7E76CR77SPW7 wasmcloud:messaging 'SUBSCRIPTION=wasmkv.>'
	wash ctl link put MC4KXRTFQJHGGH3WX3OCDZZEHAIGSUDZOG533JHJTVN44H3KPBUO4NRW VAZVC4RX54J2NVCMCW7BPCAHGGG5XZXDBXFUMDUXGESTMQEJLC3YVZWB wasmcloud:keyvalue 'URL=redis://127.0.0.1:6379'
	wash ctl link put MB3YLWPIX6IISPKTURE4DU7DVUHMGW6OBC57ZJ4CKDYZOR7SNT36E3PF VAG3QITQQ2ODAOWB5TTQSDJ53XK3SHBEIFNK4AYJ5RKAX2UNSCAPHA5M wasmcloud:httpserver 'address=0.0.0.0:8080'
	wash ctl link put MB3YLWPIX6IISPKTURE4DU7DVUHMGW6OBC57ZJ4CKDYZOR7SNT36E3PF VADNMSIML2XGO2X4TPIONTIC55R2UUQGPPDZPAVSC2QD7E76CR77SPW7 wasmcloud:messaging
	-wash ctl start provider wasmcloud.azurecr.io/kvredis:0.17.0 --skip-wait
	-wash ctl start provider wasmcloud.azurecr.io/nats_messaging:0.14.5 --skip-wait
	-wash ctl start provider wasmcloud.azurecr.io/httpserver:0.16.3 --skip-wait