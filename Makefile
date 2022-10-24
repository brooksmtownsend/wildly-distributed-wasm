start: ## Simple script to start all capability providers with links for this demo (for local use)
	wash ctl start actor ghcr.io/brooksmtownsend/dist-kv:0.2.0 --skip-wait
	wash ctl start actor ghcr.io/brooksmtownsend/todo:0.2.0 --skip-wait
	wash ctl start actor ghcr.io/brooksmtownsend/ui:0.2.0 --skip-wait
	wash ctl link put MC4KXRTFQJHGGH3WX3OCDZZEHAIGSUDZOG533JHJTVN44H3KPBUO4NRW VADNMSIML2XGO2X4TPIONTIC55R2UUQGPPDZPAVSC2QD7E76CR77SPW7 wasmcloud:messaging 'SUBSCRIPTION=wasmkv.>'
	wash ctl link put MC4KXRTFQJHGGH3WX3OCDZZEHAIGSUDZOG533JHJTVN44H3KPBUO4NRW VAZVC4RX54J2NVCMCW7BPCAHGGG5XZXDBXFUMDUXGESTMQEJLC3YVZWB wasmcloud:keyvalue 'URL=redis://127.0.0.1:6379'
	wash ctl link put MB3YLWPIX6IISPKTURE4DU7DVUHMGW6OBC57ZJ4CKDYZOR7SNT36E3PF VAG3QITQQ2ODAOWB5TTQSDJ53XK3SHBEIFNK4AYJ5RKAX2UNSCAPHA5M wasmcloud:httpserver 'address=0.0.0.0:8080'
	wash ctl link put MB3YLWPIX6IISPKTURE4DU7DVUHMGW6OBC57ZJ4CKDYZOR7SNT36E3PF VADNMSIML2XGO2X4TPIONTIC55R2UUQGPPDZPAVSC2QD7E76CR77SPW7 wasmcloud:messaging
	-wash ctl start provider wasmcloud.azurecr.io/kvredis:0.17.0 --skip-wait
	-wash ctl start provider wasmcloud.azurecr.io/nats_messaging:0.14.5 --skip-wait
	-wash ctl start provider wasmcloud.azurecr.io/httpserver:0.16.3 --skip-wait

# Replace with your constellation
CONSTELLATION ?= 374b6434-f18d-4b93-8743-bcd3089e4d5b

# This is of course a very specific demo setup, but host IDs are interchangeable
AWS ?= NDTH4AOBB7RUH3OQLXIDZL5ZA5E4VN3NTICXXK22PTSQ22HTE2Q5MY3I
AZURE ?= NCJT6FD6VLORX3YKEQYZ7YO2W4DYCCVHRYNFGNAQLLAMYYQLPDLUO5J7
COSMONICLOUD ?= NDMGWAWSL7L6RBAEIGONW2AA4D32TJTDKUSAYP5PK3UN7RNVFBXXPQYZ
GCP ?= NDT7VSWWD3E4Y54PMAIEMTOTTOQ5TUYDOHIU223URWU7H6Y76YPINWRK
ORACLE ?= NDLG26HURSYKO3AWCJ2644QVC7RCREUV6C5BOB6K6F2BZEN3XPLM24GL

start-nats: ## Run NATS on all clouds
	-wash ctl start provider wasmcloud.azurecr.io/nats_messaging:0.14.5 --skip-wait --host-id $(COSMONICLOUD) --lattice-prefix $(CONSTELLATION)
	-wash ctl start provider wasmcloud.azurecr.io/nats_messaging:0.14.5 --skip-wait --host-id $(GCP) --lattice-prefix $(CONSTELLATION)
	-wash ctl start provider wasmcloud.azurecr.io/nats_messaging:0.14.5 --skip-wait --host-id $(ORACLE) --lattice-prefix $(CONSTELLATION)
	-wash ctl start provider wasmcloud.azurecr.io/nats_messaging:0.14.5 --skip-wait --host-id $(AWS) --lattice-prefix $(CONSTELLATION)
	-wash ctl start provider wasmcloud.azurecr.io/nats_messaging:0.14.5 --skip-wait --host-id $(AZURE) --lattice-prefix $(CONSTELLATION)
	
start-redis: ## Start all redis providers with their configuration to access their cloud persistent store
	-wash ctl start provider wasmcloud.azurecr.io/kvredis:0.18.0 --skip-wait --host-id $(GCP) --lattice-prefix $(CONSTELLATION) --config-json config/gcp.json
	-wash ctl start provider wasmcloud.azurecr.io/kvredis:0.18.0 --skip-wait --host-id $(ORACLE) --lattice-prefix $(CONSTELLATION)
	-wash ctl start provider wasmcloud.azurecr.io/kvredis:0.18.0 --skip-wait --host-id $(AWS) --lattice-prefix $(CONSTELLATION) --config-json config/aws.json
	-wash ctl start provider wasmcloud.azurecr.io/kvredis:0.18.0 --skip-wait --host-id $(AZURE) --lattice-prefix $(CONSTELLATION) --config-json config/azure.json
	