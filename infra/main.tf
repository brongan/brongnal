provider "google" {
  project = var.project_id
  region  = var.region
  zone    = var.zone
}

resource "google_compute_address" "identity_static_ip" {
  name = "brongnal-identity-ip"
}

resource "google_compute_disk" "identity_data_disk" {
  name = "brongnal-identity-data"
  size = 10
  type = "pd-ssd"
  # CMEK would require a KMS Key ring and key, which we skip for this simplified version
  # unless specifically requested. We'll stick to standard encryption for now.
}

resource "google_compute_instance" "identity_service" {
  name         = "brongnal-identity"
  machine_type = "n2d-standard-2" # SEV required

  confidential_instance_config {
    enable_confidential_compute = true
  }

  shielded_instance_config {
    enable_secure_boot          = true
    enable_vtpm                 = true
    enable_integrity_monitoring = true
  }

  boot_disk {
    initialize_params {
      image = "cos-cloud/cos-stable"
    }
  }

  attached_disk {
    source      = google_compute_disk.identity_data_disk.id
    device_name = "data-disk"
  }

  network_interface {
    network = "default"
    access_config {
      nat_ip = google_compute_address.identity_static_ip.address
    }
  }

  metadata = {
    enable-oslogin = "FALSE"
    user-data      = file("${path.module}/hardening.sh")
  }

  service_account {
    scopes = ["cloud-platform"]
  }

  tags = ["brongnal-identity"]
}

resource "google_compute_firewall" "allow_https" {
  name    = "allow-https-identity"
  network = "default"

  allow {
    protocol = "tcp"
    ports    = ["443"]
  }

  source_ranges = ["0.0.0.0/0"]
  target_tags   = ["brongnal-identity"]
}
