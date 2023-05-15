default: buildspa
buildspa:
    cd ./Client && npm run build
    rm -rf ./public/
    mv ./Client/dist/ ./public/
    
