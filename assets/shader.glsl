#version 330 core

#if defined(VERTEX) // vertex shader

in vec3 pos;
in vec3 normal;
in vec2 uv;

out vec3 FragPos;
out vec3 Normal;
out vec2 TexCoords;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

void main(){
	FragPos=vec3(model*vec4(pos,1.));
	Normal=mat3(transpose(inverse(model)))*normal;
	TexCoords=uv;
	
	gl_Position=projection*view*vec4(FragPos,1.);
}

#else // fragment shader
out vec4 FragColor;

in vec3 Normal;
in vec3 FragPos;
in vec2 TexCoords;

uniform vec3 light_pos;
uniform vec3 view_pos;
uniform vec3 light_color;
uniform vec3 object_color;
uniform sampler2D diffuse_tex;
uniform sampler2D normal_tex;

vec3 ambient_color=vec3(1.)*.2;
vec3 normal_map_flat_color=vec3(.5,.5,1.);
float normal_map_intensity=.15;

mat3 cotangent_frame(vec3 normal,vec3 pos,vec2 uv){
	vec3 dp1=dFdx(pos);
	vec3 dp2=dFdy(pos);
	vec2 duv1=dFdx(uv);
	vec2 duv2=dFdy(uv);
	
	vec3 dp2perp=cross(dp2,normal);
	vec3 dp1perp=cross(normal,dp1);
	vec3 T=dp2perp*duv1.x+dp1perp*duv2.x;
	vec3 B=dp2perp*duv1.y+dp1perp*duv2.y;
	
	float invmax=inversesqrt(max(dot(T,T),dot(B,B)));
	return mat3(T*invmax,B*invmax,normal);
}

void main(){
	// normal
	vec3 normal_sample=texture(normal_tex,TexCoords).rgb;
	normal_sample=mix(normal_map_flat_color,normal_sample,normal_map_intensity);
	mat3 tbn=cotangent_frame(Normal,FragPos,TexCoords);
	vec3 norm=normalize(tbn*(normal_sample*2.-1.));
	
	// diffuse
	vec3 light_dir=normalize(light_pos-FragPos);
	float diff=max(dot(norm,light_dir),0.);
	vec3 diffuse=diff*light_color*texture(diffuse_tex,TexCoords).rgb;
	
	// specular
	float specular_strength=.5;
	vec3 view_dir=normalize(view_pos-FragPos);
	vec3 reflect_dir=reflect(-light_dir,norm);
	float spec=pow(max(dot(view_dir,reflect_dir),0.),32);
	vec3 specular=specular_strength*spec*light_color;
	
	vec3 result=(ambient_color+diffuse+specular)*object_color;
	FragColor=vec4(result,1.);
}

#endif