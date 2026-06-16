pub(super) fn routes(custom_routes: &[String]) -> Vec<String> {
    let mut routes = vec!["/".to_string()];
    for route in custom_routes {
        if !routes.iter().any(|existing| existing == route) {
            routes.push(route.clone());
        }
    }
    routes
}
