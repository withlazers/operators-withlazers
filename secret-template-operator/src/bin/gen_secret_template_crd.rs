use kube::CustomResourceExt;
use secret_template_operator::crd::SecretTemplate;

fn main() {
    SecretTemplate::crd();
    print!("{}", serde_yaml::to_string(&SecretTemplate::crd()).unwrap());
}
