pipeline {
  agent {
    kubernetes {
      cloud 'kubernetes'
      nodeSelector 'purpose=runner'
      serviceAccount 'jenkins-agent-sa'
      yaml '''spec:
    tolerations:
    - key: "purpose"
      operator: "Equal"
      value: "runner"
      effect: "NoSchedule"
    containers:
    - name: jenkins-standard-agent
      image: asia-southeast2-docker.pkg.dev/ihc-dto-corp/devops/jenkins-agent:latest
      imagePullPolicy: Always
      command:
      - cat
      tty: true
      resources:
        requests:
          cpu: 100m
          memory: 256Mi
        limits:
          cpu: 200
          memory: 512Mi
    - name: kaniko
      image: gcr.io/kaniko-project/executor:v1.23.1-debug
      command:
      - cat
      tty: true
      resources:
        requests:
          cpu: 500m
          memory: 512Mi
        limits:
          cpu: 1000m
          memory: 2Gi
    - name: krane
      image: gcr.io/go-containerregistry/krane:debug
      command:
      - cat
      tty: true
      resources:
        requests:
          cpu: 100m
          memory: 128Mi
        limits:
          cpu: 200m
          memory: 256Mi'''
    }
  }
  options {
    gitLabConnection('gitlab-connection-ihc')
  }

  // Stages status
  environment {
    DOCKER_REGISTRY = 'asia-southeast2-docker.pkg.dev/ihc-dto-corp/devops'
    IMAGE_NAME = 'officer'
    SHORT_COMMIT_HASH = sh(script: 'git rev-parse --short HEAD', returnStdout: true).trim()
    VERSION_NUMBER = 'unknown'
  }
  stages {
    stage('prerun') {
      steps {
        container('jenkins-standard-agent') {
          script {
            try {
              sh "gcloud auth list"
              sh "pwd"
              sh "ls"
              sh "gcloud auth configure-docker asia-southeast2-docker.pkg.dev"
              // End of custom command
            } catch (Exception e){
              throw e
            }
          }
        }
      }
    }
    stage('build') {
      steps {
        container('kaniko') {
          script {
            try {
            updateGitlabCommitStatus name: 'build', state: 'running'
             sh """
                /kaniko/executor --dockerfile=`pwd`/Dockerfile \
                --context=`pwd` \
                --destination=${DOCKER_REGISTRY}/${IMAGE_NAME}:${SHORT_COMMIT_HASH} \
                --tar-path=image.tar \
                --cache=true \
                --no-push
                """
              // End of custom command
            } catch (Exception e){
              updateGitlabCommitStatus name: 'build', state: 'failed'
              throw e
            }
          }
        }
      }
      post {
        success {
          updateGitlabCommitStatus name: 'build', state: 'success'
        }
        aborted {
          updateGitlabCommitStatus name: 'build', state: 'canceled'
        }
      }
    }
    stage('test') {
      steps {
        container('jenkins-standard-agent') {
          script {
            try {
              updateGitlabCommitStatus name: 'test', state: 'running'
              // Custom command here: 
              sh 'echo "Skipping test..."'
              // End of custom command
            } catch (Exception e){
              updateGitlabCommitStatus name: 'test', state: 'failed'
              throw e
            }
          }
        }
      }
      post {
        success {
          updateGitlabCommitStatus name: 'test', state: 'success'
        }
        aborted {
          updateGitlabCommitStatus name: 'test', state: 'canceled'
        }
      }
    }
    stage('push') {
      steps {
        container('krane') {
          script {
            try {
              updateGitlabCommitStatus name: 'push', state: 'running'
              // Custom command here: 
                // Build Docker image and tag it
              if (env.BRANCH_NAME == 'main'){
                sh """
                  krane push image.tar ${DOCKER_REGISTRY}/${IMAGE_NAME}:${SHORT_COMMIT_HASH}
                  krane push image.tar ${DOCKER_REGISTRY}/${IMAGE_NAME}:latest
                """
              } else if (env.GIT_TAG_NAME != null && env.GIT_TAG_NAME != ''){
                sh """
                  krane push image.tar ${DOCKER_REGISTRY}/${IMAGE_NAME}:${env.GIT_TAG_NAME}
                """
              }else if (env.BRANCH_NAME.startsWith('release/')){
                // Setting image tag
                def branchName = env.BRANCH_NAME ?: 'unknown'
                // Initialize version number
                def versionNumber = 'unknown'
                // Print BRANCH_NAME for debugging
                sh 'echo "BRANCH_NAME: ${BRANCH_NAME}"'
                // Check if branch name starts with 'release/'
                if (branchName.startsWith('release/')) {
                    versionNumber = branchName.split('/').last()
                }
                // Print version number
                echo "Version Number: ${versionNumber}"
                sh """
                  krane push image.tar ${DOCKER_REGISTRY}/${IMAGE_NAME}:${versionNumber}
                """
              } else {
                sh """
                  krane push image.tar ${DOCKER_REGISTRY}/${IMAGE_NAME}:${SHORT_COMMIT_HASH}
                """
              }
              sh 'echo "Push finished!"'
              // End of custom command
            } catch (Exception e){
              updateGitlabCommitStatus name: 'push', state: 'failed'
              throw e
            }
          }
        }
      }
      post {
        success {
          updateGitlabCommitStatus name: 'push', state: 'success'
        }
        aborted {
          updateGitlabCommitStatus name: 'push', state: 'canceled'
        }
      }
    }
    stage('deploy') {
      // Conditional branch
      when {
        expression {
          env.BRANCH_NAME == 'main' || env.BRANCH_NAME == 'development' || (env.GIT_TAG_NAME != null && env.GIT_TAG_NAME != '')
        }
      }
      steps {
        container('jenkins-standard-agent') {
          script {
            try {
              updateGitlabCommitStatus name: 'deploy', state: 'running'
              // Custom command here:
              if (env.BRANCH_NAME == 'main'){
                // gcloud config set auth/impersonate_service_account jenkins-agent-gsa@medinesia-prod.iam.gserviceaccount.com
                // sh """
                //   gcloud container clusters get-credentials medinesia-prod --region asia-southeast2 --project medinesia-prod && \
                //   kubectl get nodes
                // """
              } else if (env.BRANCH_NAME == 'development'){
                // sh """
                //   gcloud container clusters get-credentials medinesia-dev --zone asia-southeast2-c --project medinesia-dev && \
                //   kubectl set image deployment/sample-app sample=${DOCKER_REGISTRY}/${IMAGE_NAME}:${SHORT_COMMIT_HASH} -n sample
                // """
              }else{
                sh 'echo "Do something else!"'
              }
              // End of custom command
            } catch (Exception e){
              updateGitlabCommitStatus name: 'deploy', state: 'failed'
              throw e
            }
          }
        }
      }
      post {
        success {
          updateGitlabCommitStatus name: 'deploy', state: 'success'
        }
        aborted {
          updateGitlabCommitStatus name: 'deploy', state: 'canceled'
        }
      }
    }
  }
}
