. "$PSScriptRoot/Private/Base64Url.ps1"

#1 Start Mongo manually: 'docker run -p 27017:27017 -d --name picky-mongo library/mongo:4.1-bionic'
#2 Set the environnement variable of the build if they are not set
#3 Run Picky Server with Clion
#4 Run Pester './PickyDebug.Test'
Describe 'Picky tests' {
	BeforeAll {
		$picky_url = "http://127.0.0.1:12345"
		$picky_realm = "WaykDen"
		$picky_authority = "${picky_realm} Authority"
		$picky_api_key = "secret"
		$picky_backend = "mongodb"
		$picky_database_url = "mongodb://picky-mongo:27017"

		& 'docker' 'stop' 'picky-server'
		& 'docker' 'rm' 'picky-server'
	}

	It 'checks health' {
		Write-Host "$picky_url/health"
		$request = Invoke-WebRequest -Uri $picky_url/health -Method 'GET' -ContentType 'text/plain'
		$request.StatusCode | Should -Be 200
	}

	It 'gets CA chain' {
		$authority_base64 = ConvertTo-Base64Url $picky_authority
		$contents = Invoke-RestMethod -Uri $picky_url/chain/$authority_base64 -Method 'GET' `
			-ContentType 'text/plain'

		$ca_chain = @()
		# https://stackoverflow.com/questions/45884754/powershell-extract-multiple-occurrences-in-multi-lines
		$contents | Select-String  -Pattern '(?smi)^-{2,}BEGIN CERTIFICATE-{2,}.*?-{2,}END CERTIFICATE-{2,}' `
			-Allmatches | ForEach-Object {$_.Matches} | ForEach-Object { $ca_chain += $_.Value }

		$ca_chain.Count | Should -Be 2
		Set-Content -Value $ca_chain[0] -Path "$TestDrive/intermediate_ca.pem"
		Set-Content -Value $ca_chain[1] -Path "$TestDrive/root_ca.pem"

		$root_ca = New-Object System.Security.Cryptography.X509Certificates.X509Certificate2("$TestDrive/root_ca.pem")
		$intermediate_ca = New-Object System.Security.Cryptography.X509Certificates.X509Certificate2("$TestDrive/intermediate_ca.pem")

		$intermediate_ca.Subject | Should -Be "CN=${picky_realm} Authority"
		$intermediate_ca.Issuer | Should -Be "CN=${picky_realm} Root CA"

		$root_ca.Subject | Should -Be "CN=${picky_realm} Root CA"
		$root_ca.Issuer | Should -Be "CN=${picky_realm} Root CA"
	}

	It 'signs certificates JSON with CA and CSR' {
		# https://stackoverflow.com/questions/48196350/generate-and-sign-certificate-request-using-pure-net-framework
		# https://www.powershellgallery.com/packages/SelfSignedCertificate/0.0.4/Content/SelfSignedCertificate.psm1

		$key_size = 2048
		$subject = "CN=test.${picky_realm}"
		$rsa_key = [System.Security.Cryptography.RSA]::Create($key_size)

		$certRequest = [System.Security.Cryptography.X509Certificates.CertificateRequest]::new(
				$subject, $rsa_key,
				[System.Security.Cryptography.HashAlgorithmName]::SHA256,
				[System.Security.Cryptography.RSASignaturePadding]::Pkcs1)

		$csr_der = $certRequest.CreateSigningRequest()

		$sb = [System.Text.StringBuilder]::new()
		$csr_base64 = [Convert]::ToBase64String($csr_der)

		$offset = 0
		$line_length = 64
		$sb.AppendLine("-----BEGIN CERTIFICATE REQUEST-----")
		while ($offset -lt $csr_base64.Length) {
			$line_end = [Math]::Min($offset + $line_length, $csr_base64.Length)
			$sb.AppendLine($csr_base64.Substring($offset, $line_end - $offset))
			$offset = $line_end
		}
		$sb.AppendLine("-----END CERTIFICATE REQUEST-----")
		$csr_pem = $sb.ToString()

		Write-Host $csr_pem

		Set-Content -Value $csr_pem -Path "$TestDrive/test.csr"
		$csr = Get-Content "$TestDrive/test.csr" | Out-String

		$headers = @{
			"Authorization" = "Bearer $picky_api_key"
		}

		$payload = [PSCustomObject]@{
			ca="$picky_authority"
			csr="$csr"
		} | ConvertTo-Json

		$Bytes = [System.Text.Encoding]::Unicode.GetBytes($csr)
		$body = [Convert]::ToBase64String($Bytes)

		Write-Host $body

		Write-Host $payload

		$cert = Invoke-RestMethod -Uri $picky_url/signcert/ -Method 'POST' `
			-Headers $headers `
			-ContentType 'application/json' `
			-Body $payload

		Set-Content -Value $cert -Path "$TestDrive/test.crt"
		$leaf_cert = New-Object System.Security.Cryptography.X509Certificates.X509Certificate2("$TestDrive/test.crt")

		Write-Host $leaf_cert
		$leaf_cert.Subject | Should -Be "CN=test.${picky_realm}"
		$leaf_cert.Issuer | Should -Be "CN=${picky_realm} Authority"
	}

	It 'signs certificates With CSR as base64' {
		# https://stackoverflow.com/questions/48196350/generate-and-sign-certificate-request-using-pure-net-framework
		# https://www.powershellgallery.com/packages/SelfSignedCertificate/0.0.4/Content/SelfSignedCertificate.psm1

		$key_size = 2048
		$subject = "CN=test.${picky_realm}"
		$rsa_key = [System.Security.Cryptography.RSA]::Create($key_size)

		$certRequest = [System.Security.Cryptography.X509Certificates.CertificateRequest]::new(
				$subject, $rsa_key,
				[System.Security.Cryptography.HashAlgorithmName]::SHA256,
				[System.Security.Cryptography.RSASignaturePadding]::Pkcs1)

		$csr_der = $certRequest.CreateSigningRequest()

		$sb = [System.Text.StringBuilder]::new()
		$csr_base64 = [Convert]::ToBase64String($csr_der)

		$offset = 0
		$line_length = 64
		$sb.AppendLine("-----BEGIN CERTIFICATE REQUEST-----")
		while ($offset -lt $csr_base64.Length) {
			$line_end = [Math]::Min($offset + $line_length, $csr_base64.Length)
			$sb.AppendLine($csr_base64.Substring($offset, $line_end - $offset))
			$offset = $line_end
		}
		$sb.AppendLine("-----END CERTIFICATE REQUEST-----")
		$csr_pem = $sb.ToString()

		Write-Host $csr_pem

		Set-Content -Value $csr_pem -Path "$TestDrive/test.csr"
		$csr = Get-Content "$TestDrive/test.csr" | Out-String

		$headers = @{
			"Authorization" = "Bearer $picky_api_key"
			"Content-Transfer-Encoding" = "base64"
			"Content-Disposition" = "attachment"
		}

		Write-Host $body

		Write-Host $payload

		$cert = Invoke-RestMethod -Uri $picky_url/signcert/ -Method 'POST' `
                -ContentType 'application/pkcs10' `
                -Headers $headers `
                -Body $csr

		Set-Content -Value $cert -Path "$TestDrive/test.crt"
		$leaf_cert = New-Object System.Security.Cryptography.X509Certificates.X509Certificate2("$TestDrive/test.crt")

		Write-Host $leaf_cert
		$leaf_cert.Subject | Should -Be "CN=test.${picky_realm}"
		$leaf_cert.Issuer | Should -Be "CN=${picky_realm} Authority"
	}

	It 'signs certificates With CSR as binary' {
		# https://stackoverflow.com/questions/48196350/generate-and-sign-certificate-request-using-pure-net-framework
		# https://www.powershellgallery.com/packages/SelfSignedCertificate/0.0.4/Content/SelfSignedCertificate.psm1

		$key_size = 2048
		$subject = "CN=test.${picky_realm}"
		$rsa_key = [System.Security.Cryptography.RSA]::Create($key_size)

		$certRequest = [System.Security.Cryptography.X509Certificates.CertificateRequest]::new(
				$subject, $rsa_key,
				[System.Security.Cryptography.HashAlgorithmName]::SHA256,
				[System.Security.Cryptography.RSASignaturePadding]::Pkcs1)

		$csr_der = $certRequest.CreateSigningRequest()

		$sb = [System.Text.StringBuilder]::new()
		$csr_base64 = [Convert]::ToBase64String($csr_der)

		$offset = 0
		$line_length = 64
		$sb.AppendLine("-----BEGIN CERTIFICATE REQUEST-----")
		while ($offset -lt $csr_base64.Length) {
			$line_end = [Math]::Min($offset + $line_length, $csr_base64.Length)
			$sb.AppendLine($csr_base64.Substring($offset, $line_end - $offset))
			$offset = $line_end
		}
		$sb.AppendLine("-----END CERTIFICATE REQUEST-----")
		$csr_pem = $sb.ToString()

		Write-Host $csr_pem

		Set-Content -Value $csr_pem -Path "$TestDrive/test.csr"
		$csr = Get-Content "$TestDrive/test.csr" | Out-String

		$csr = [Convert]::FromBase64String($csr_base64)

		$headers = @{
			"Authorization" = "Bearer $picky_api_key"
			"Content-Transfer-Encoding" = "binary"
			"Content-Disposition" = "attachment"
		}

		$cert = Invoke-RestMethod -Uri $picky_url/signcert/ -Method 'POST' `
                -ContentType 'application/pkcs10' `
                -Headers $headers `
                -Body $csr

		Set-Content -Value $cert -Path "$TestDrive/test.crt"
		$leaf_cert = New-Object System.Security.Cryptography.X509Certificates.X509Certificate2("$TestDrive/test.crt")

		Write-Host $leaf_cert
		$leaf_cert.Subject | Should -Be "CN=test.${picky_realm}"
		$leaf_cert.Issuer | Should -Be "CN=${picky_realm} Authority"
	}


	It 'signs certificates Who failed, Send without Content-Transfert-Encoding' {
		# https://stackoverflow.com/questions/48196350/generate-and-sign-certificate-request-using-pure-net-framework
		# https://www.powershellgallery.com/packages/SelfSignedCertificate/0.0.4/Content/SelfSignedCertificate.psm1

		$key_size = 2048
		$subject = "CN=test.${picky_realm}"
		$rsa_key = [System.Security.Cryptography.RSA]::Create($key_size)

		$certRequest = [System.Security.Cryptography.X509Certificates.CertificateRequest]::new(
				$subject, $rsa_key,
				[System.Security.Cryptography.HashAlgorithmName]::SHA256,
				[System.Security.Cryptography.RSASignaturePadding]::Pkcs1)

		$csr_der = $certRequest.CreateSigningRequest()

		$sb = [System.Text.StringBuilder]::new()
		$csr_base64 = [Convert]::ToBase64String($csr_der)

		$offset = 0
		$line_length = 64
		$sb.AppendLine("-----BEGIN CERTIFICATE REQUEST-----")
		while ($offset -lt $csr_base64.Length) {
			$line_end = [Math]::Min($offset + $line_length, $csr_base64.Length)
			$sb.AppendLine($csr_base64.Substring($offset, $line_end - $offset))
			$offset = $line_end
		}
		$sb.AppendLine("-----END CERTIFICATE REQUEST-----")
		$csr_pem = $sb.ToString()

		Set-Content -Value $csr_pem -Path "$TestDrive/test.csr"
		$csr = Get-Content "$TestDrive/test.csr" | Out-String

		$headers = @{
			"Authorization" = "Bearer $picky_api_key"
			"Content-Disposition" = "attachment"
		}

		try{
			Invoke-RestMethod -Uri $picky_url/signcert/ -Method 'POST' `
			-ContentType 'application/pkcs10' `
			-Headers $headers `
			-Body $csr
		}
		catch{
			Write-Host $_
			return;
		}

		throw "This test sould catch the web-request"
	}
}