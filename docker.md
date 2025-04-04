Chào bạn! Bạn muốn tìm hiểu về tất cả các câu lệnh Docker? Đây là một danh sách khá đầy đủ các câu lệnh Docker phổ biến và quan trọng, được phân loại để bạn dễ dàng tham khảo.

**Lưu ý quan trọng:**

- **Cấu trúc câu lệnh Docker:** Hầu hết các câu lệnh Docker có cấu trúc chung là: `docker [OPTIONS] COMMAND [ARGUMENTS]`
  - `docker`: Lệnh gọi chương trình Docker CLI.
  - `[OPTIONS]`: Các tùy chọn để điều chỉnh hành vi của lệnh (ví dụ: `-p`, `-v`, `--name`).
  - `COMMAND`: Động từ chỉ hành động bạn muốn thực hiện (ví dụ: `run`, `build`, `images`, `ps`).
  - `[ARGUMENTS]`: Các đối số cụ thể cho lệnh (ví dụ: tên image, tên container, đường dẫn).
- **`docker --help` hoặc `docker COMMAND --help`:** Đây là bạn thân nhất của bạn! Sử dụng `--help` để xem hướng dẫn chi tiết về bất kỳ lệnh Docker nào, bao gồm tất cả các tùy chọn có sẵn. Ví dụ: `docker run --help` hoặc `docker image build --help`.

**Phân loại các câu lệnh Docker:**

Để dễ dàng tìm kiếm và học tập, chúng ta sẽ phân loại các câu lệnh Docker theo chức năng chính của chúng:

**1. Quản lý Images (Ảnh):**

- **`docker images` hoặc `docker image ls`:** Liệt kê tất cả các images hiện có trên máy của bạn.
  - **`docker images -a` hoặc `docker image ls -a`:** Liệt kê tất cả images, bao gồm cả các intermediate images (ảnh trung gian).
  - **`docker images <tên_image>` hoặc `docker image ls <tên_image>`:** Liệt kê thông tin của image cụ thể.
  - **`docker images --filter "dangling=true"` hoặc `docker image ls --filter "dangling=true"`:** Liệt kê các "dangling images" (images không còn tag và không được tham chiếu bởi container nào).
- **`docker pull <tên_image>[:tag]`:** Tải một image từ Docker Hub hoặc registry khác về máy của bạn.
  - Ví dụ: `docker pull ubuntu:latest` (tải image Ubuntu phiên bản mới nhất).
  - Ví dụ: `docker pull my-registry.com/my-namespace/my-app:v1.0` (tải image từ registry riêng).
- **`docker build [OPTIONS] PATH | URL | -`:** Xây dựng một image Docker từ một Dockerfile.
  - **`docker build -t <tên_image>:<tag> .`:** Xây dựng image từ Dockerfile trong thư mục hiện tại (`.`), gắn tag `tên_image:tag`.
  - **`docker build -f <đường_dẫn_Dockerfile> -t <tên_image>:<tag> .`:** Xây dựng image từ Dockerfile ở đường dẫn cụ thể.
  - **`docker build --build-arg <BIẾN>=<GIÁ_TRỊ> ...`:** Truyền các biến build-time vào Dockerfile.
- **`docker push <tên_image>:<tag>`:** Đẩy một image lên Docker Hub hoặc registry khác. Bạn cần `docker login` trước.
- **`docker rmi <tên_image_hoặc_ID_image> ...` hoặc `docker image rm <tên_image_hoặc_ID_image> ...`:** Xóa một hoặc nhiều image.
  - **`docker rmi $(docker images -q --filter "dangling=true")`:** Xóa tất cả dangling images.
- **`docker image inspect <tên_image_hoặc_ID_image>`:** Xem thông tin chi tiết về một image (JSON format).
- **`docker image history <tên_image>`:** Xem lịch sử các layers của một image.
- **`docker image tag <SOURCE_IMAGE>[:TAG] <TARGET_IMAGE>[:TAG]`:** Tạo một tag mới cho một image hiện có.
  - Ví dụ: `docker image tag my-app:v1 my-app:latest` (tạo tag "latest" cho image "my-app:v1").
- **`docker image prune [OPTIONS]`:** Xóa các dangling images và unused images (theo tùy chọn).
  - **`docker image prune -a`:** Xóa tất cả unused images, không chỉ dangling.
- **`docker image save [OPTIONS] <IMAGE> [IMAGE...]`:** Lưu một hoặc nhiều images vào một file tar.
  - **`docker image save -o my-image.tar my-app:latest`:** Lưu image "my-app:latest" vào file "my-image.tar".
- **`docker image load [OPTIONS] [FILE | -]`:** Tải images từ một file tar đã lưu bằng `docker image save`.
  - **`docker image load -i my-image.tar`:** Tải images từ file "my-image.tar".

**2. Quản lý Containers (Bộ chứa):**

- **`docker run [OPTIONS] <IMAGE> [COMMAND] [ARG...]`:** Tạo và chạy một container từ một image. Đây là lệnh quan trọng nhất.
  - **`docker run -it --rm ubuntu:latest /bin/bash`:** Chạy container tương tác từ image Ubuntu, mở shell bash, và tự động xóa container sau khi thoát (`--rm`).
  - **`docker run -d -p 8080:80 --name my-web-app nginx:latest`:** Chạy container detached (`-d`), map port 8080 host tới port 80 container (`-p`), đặt tên container là "my-web-app" (`--name`), từ image Nginx.
  - **`-p <host_port>:<container_port>`:** Map port từ host vào container.
  - **`-v <host_path>:<container_path>`:** Mount một volume (chia sẻ thư mục) từ host vào container.
  - **`--name <tên_container>`:** Đặt tên cho container.
  - **`-d`:** Chạy container ở chế độ detached (background).
  - **`-it`:** Chạy container ở chế độ interactive và TTY (cho phép tương tác với shell).
  - **`--rm`:** Tự động xóa container khi nó dừng.
  - **`--env <BIẾN>=<GIÁ_TRỊ>` hoặc `-e <BIẾN>=<GIÁ_TRỊ>`:** Thiết lập biến môi trường trong container.
  - **`--network <tên_network>`:** Kết nối container vào một network cụ thể.
  - **`--restart <policy>`:** Thiết lập chính sách restart cho container (ví dụ: `always`, `on-failure`, `no`).
  - **`--privileged`:** Chạy container với quyền privileged (cẩn thận khi sử dụng).
- **`docker ps` hoặc `docker container ls`:** Liệt kê các container đang chạy.
  - **`docker ps -a` hoặc `docker container ls -a`:** Liệt kê tất cả các container (đang chạy và đã dừng).
  - **`docker ps -q` hoặc `docker container ls -q`:** Liệt kê chỉ ID của các container đang chạy.
  - **`docker ps --filter "status=exited"` hoặc `docker container ls --filter "status=exited"`:** Liệt kê các container đã dừng.
- **`docker stop <tên_container_hoặc_ID_container> ...`:** Dừng một hoặc nhiều container đang chạy.
- **`docker start <tên_container_hoặc_ID_container> ...`:** Khởi động lại một hoặc nhiều container đã dừng.
- **`docker restart <tên_container_hoặc_ID_container> ...`:** Restart một hoặc nhiều container.
- **`docker kill <tên_container_hoặc_ID_container> ...`:** Buộc dừng một hoặc nhiều container (gửi tín hiệu SIGKILL).
- **`docker rm <tên_container_hoặc_ID_container> ...` hoặc `docker container rm <tên_container_hoặc_ID_container> ...`:** Xóa một hoặc nhiều container đã dừng.
  - **`docker rm $(docker ps -aq)`:** Xóa tất cả các container đã dừng.
  - **`docker rm -f <tên_container_hoặc_ID_container> ...`:** Buộc xóa một container đang chạy (dữ liệu có thể bị mất).
- **`docker exec [OPTIONS] <tên_container_hoặc_ID_container> <COMMAND> [ARG...]`:** Chạy một lệnh bên trong một container đang chạy.
  - **`docker exec -it <tên_container> /bin/bash`:** Mở shell bash bên trong container để tương tác.
- **`docker logs [OPTIONS] <tên_container_hoặc_ID_container>`:** Xem logs của một container.
  - **`docker logs -f <tên_container>`:** Theo dõi logs theo thời gian thực (giống `tail -f`).
- **`docker inspect <tên_container_hoặc_ID_container>`:** Xem thông tin chi tiết về một container (JSON format).
- **`docker stats [OPTIONS] [container...]`:** Hiển thị thống kê tài nguyên (CPU, memory, network, I/O) của một hoặc nhiều container đang chạy.
- **`docker top <tên_container_hoặc_ID_container>`:** Hiển thị các process đang chạy bên trong container (giống lệnh `top` trong Linux).
- **`docker rename <OLD_NAME> <NEW_NAME>`:** Đổi tên một container.
- **`docker pause <tên_container_hoặc_ID_container> ...`:** Tạm dừng một hoặc nhiều container.
- **`docker unpause <tên_container_hoặc_ID_container> ...`:** Tiếp tục chạy một hoặc nhiều container đã tạm dừng.
- **`docker container prune [OPTIONS]`:** Xóa các container đã dừng (theo tùy chọn).

**3. Quản lý Volumes (Ổ đĩa dữ liệu):**

- **`docker volume create [OPTIONS] [VOLUME]`:** Tạo một volume mới.
  - **`docker volume create my-data-volume`:** Tạo volume tên "my-data-volume".
- **`docker volume ls`:** Liệt kê tất cả các volumes.
- **`docker volume inspect <VOLUME> ...`:** Xem thông tin chi tiết về một hoặc nhiều volumes (JSON format).
- **`docker volume rm <VOLUME> ...`:** Xóa một hoặc nhiều volumes.
- **`docker volume prune [OPTIONS]`:** Xóa các unused volumes.

**4. Quản lý Networks (Mạng):**

- **`docker network create [OPTIONS] <NETWORK_NAME>`:** Tạo một network mới.
  - **`docker network create my-custom-network`:** Tạo network tên "my-custom-network".
  - **`docker network create --driver bridge my-bridge-network`:** Tạo network bridge.
- **`docker network ls`:** Liệt kê tất cả các networks.
- **`docker network inspect <NETWORK_NAME_hoặc_ID_network> ...`:** Xem thông tin chi tiết về một hoặc nhiều networks (JSON format).
- **`docker network rm <NETWORK_NAME_hoặc_ID_network> ...`:** Xóa một hoặc nhiều networks.
- **`docker network connect <NETWORK_NAME> <CONTAINER>`:** Kết nối một container vào một network.
- **`docker network disconnect <NETWORK_NAME> <CONTAINER>`:** Ngắt kết nối một container khỏi một network.
- **`docker network prune [OPTIONS]`:** Xóa các unused networks.

**5. Docker Hub và Registry:**

- **`docker login [OPTIONS] [SERVER]`:** Đăng nhập vào Docker Hub hoặc một registry khác.
- **`docker logout [SERVER]`:** Đăng xuất khỏi Docker Hub hoặc một registry khác.
- **`docker search [OPTIONS] TERM`:** Tìm kiếm images trên Docker Hub.

**6. Docker Compose (Quản lý ứng dụng đa container):**

- **`docker compose up [OPTIONS]`:** Xây dựng, (re)create, start và attach containers cho một ứng dụng được định nghĩa trong `docker-compose.yml`.
  - **`docker compose up -d`:** Chạy ứng dụng ở chế độ detached (background).
  - **`docker compose up --build`:** Buộc build lại images trước khi chạy.
- **`docker compose down [OPTIONS]`:** Dừng và xóa containers, networks, volumes được định nghĩa trong `docker-compose.yml`.
- **`docker compose ps`:** Liệt kê trạng thái của các containers được định nghĩa trong `docker-compose.yml`.
- **`docker compose logs [OPTIONS] [SERVICE...]`:** Xem logs của các services (container) được định nghĩa trong `docker-compose.yml`.
- **`docker compose exec [OPTIONS] SERVICE COMMAND [ARG...]`:** Chạy một lệnh bên trong một service (container) được định nghĩa trong `docker-compose.yml`.
- **`docker compose stop [SERVICE...]`:** Dừng các services được định nghĩa trong `docker-compose.yml`.
- **`docker compose start [SERVICE...]`:** Khởi động lại các services được định nghĩa trong `docker-compose.yml`.
- **`docker compose restart [SERVICE...]`:** Restart các services được định nghĩa trong `docker-compose.yml`.
- **`docker compose build [SERVICE...]`:** Build hoặc rebuild images cho các services được định nghĩa trong `docker-compose.yml`.

**7. Thông tin hệ thống và Utilities:**

- **`docker info`:** Hiển thị thông tin hệ thống Docker.
- **`docker version`:** Hiển thị thông tin phiên bản Docker Client và Docker Engine.
- **`docker system df`:** Hiển thị thông tin sử dụng disk space của Docker.
- **`docker system prune [OPTIONS]`:** Xóa các unused data của Docker (images, containers, volumes, networks).
  - **`docker system prune -a`:** Xóa tất cả unused data, bao gồm cả images không được sử dụng bởi bất kỳ container nào.
- **`docker events [OPTIONS]`:** Xem các sự kiện Docker theo thời gian thực.

**8. Docker Buildx (Mở rộng khả năng Build):**

- **`docker buildx build [OPTIONS] PATH | URL | -`:** Tương tự `docker build` nhưng với nhiều tính năng nâng cao hơn, như multi-platform builds, build cache sharing, etc.
- **`docker buildx create [OPTIONS] [NAME]`:** Tạo một builder instance mới.
- **`docker buildx use [NAME]`:** Chọn builder instance để sử dụng.
- **`docker buildx inspect [NAME]`:** Xem thông tin về một builder instance.
- **`docker buildx ls`:** Liệt kê các builder instances.
- **`docker buildx rm [NAME]`:** Xóa một builder instance.

**Lời khuyên:**

- **Thực hành là chìa khóa:** Hãy thử nghiệm các câu lệnh này trực tiếp trên máy của bạn để hiểu rõ hơn cách chúng hoạt động.
- **Sử dụng `--help` thường xuyên:** Khi bạn không chắc chắn về một lệnh hoặc tùy chọn nào đó, hãy sử dụng `--help`.
- **Tìm hiểu Docker Compose:** Nếu bạn làm việc với các ứng dụng phức tạp hơn, Docker Compose sẽ giúp bạn quản lý dễ dàng hơn.
- **Tham khảo tài liệu chính thức:** [https://docs.docker.com/](https://docs.docker.com/) là nguồn tài liệu tốt nhất để tìm hiểu sâu hơn về Docker.

Chúc bạn học Docker thành công! Nếu bạn có bất kỳ câu hỏi cụ thể nào, đừng ngần ngại hỏi nhé!
