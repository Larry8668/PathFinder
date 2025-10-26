import { Link } from "react-router-dom"
import { useForm } from "react-hook-form"
import { useNavigate } from "react-router-dom";


function Name(){

const navigate = useNavigate();

const {
    register,
    handleSubmit,
    formState: { errors },
  } = useForm()

  const onSubmit = (data) => {
    localStorage.setItem("name", JSON.stringify(data))
    // console.log(localStorage.getItem("name"))
    navigate("/About");
}

    return(
        <div className="flex flex-col justify-center items-center gap-5 h-[100vh] bg-[#3D3C3C] font-sans">
            <div className="text-4xl">Enter Your Name</div>
            <div>
                <form onSubmit={handleSubmit(onSubmit)}>
                    <input
                    className="bg-white rounded-md text-2xl text-center"
                    defaultValue="" {...register("name")} type='text' placeholder="Type"/>
                </form>
            </div>
        <Link to='/About'><button type='submit' className="absolute bottom-3 right-5 flex flex-row justify-center py-1 px-3 bg-black rounded-md text-center text-white ">next</button></Link>

        </div>
    )
}

export default Name